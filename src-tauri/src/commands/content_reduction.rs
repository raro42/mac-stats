//! Content reduction (truncation/summarization) and skill/JS execution helpers.
//!
//! Extracted from `ollama.rs` to keep the orchestrator focused.

use std::io::Write;
use std::process::Command;

use crate::commands::ollama_chat::send_ollama_chat_messages;

/// Heuristic: chars to tokens (conservative).
pub(crate) const CHARS_PER_TOKEN: usize = 4;

/// Reserve tokens for model reply and wrapper text.
const RESERVE_TOKENS: u32 = 512;

/// When over limit by at most 1/this fraction, truncate only (no summarization) to avoid extra Ollama call.
const TRUNCATE_ONLY_THRESHOLD_DENOM: u32 = 4;

/// Truncate at last newline or space before max_chars so we don't cut mid-word. O(max_chars).
pub(crate) fn truncate_at_boundary(body: &str, max_chars: usize) -> String {
    let mut last_break = max_chars;
    let mut broke_early = false;
    for (i, c) in body.chars().enumerate() {
        if i >= max_chars {
            broke_early = true;
            break;
        }
        if c == '\n' || c == ' ' {
            last_break = i + 1;
        }
    }
    if !broke_early {
        return body.to_string();
    }
    body.chars().take(last_break).collect()
}

/// Reduce fetched page content to fit the model context: summarize via Ollama if needed, else truncate.
/// Uses byte-length heuristic for fast path and "slightly over" path to avoid full char count; only
/// when summarization is needed do we count chars for logging.
pub(crate) async fn reduce_fetched_content_to_fit(
    body: &str,
    context_size_tokens: u32,
    estimated_used_tokens: u32,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Result<String, String> {
    use tracing::info;

    let max_tokens_for_body = context_size_tokens
        .saturating_sub(RESERVE_TOKENS)
        .saturating_sub(estimated_used_tokens);
    let max_chars = (max_tokens_for_body as usize).saturating_mul(CHARS_PER_TOKEN);

    // Fast path: cheap byte heuristic (len/4 >= char_count/4 for UTF-8). Avoids char count when body fits.
    let body_tokens_upper = body.len() / CHARS_PER_TOKEN;
    if body_tokens_upper <= max_tokens_for_body as usize {
        return Ok(body.to_string());
    }

    // Slightly over: within 25% of limit → truncate only, no summarization (saves one Ollama round-trip).
    let threshold = max_tokens_for_body + (max_tokens_for_body / TRUNCATE_ONLY_THRESHOLD_DENOM);
    if body_tokens_upper <= threshold as usize {
        let truncated = truncate_at_boundary(body, max_chars);
        return Ok(format!(
            "{} (content truncated due to context limit)",
            truncated.trim_end()
        ));
    }

    // Way over: summarization path. Compute exact token estimate only for logging.
    let body_tokens_est = body.chars().count() / CHARS_PER_TOKEN;
    info!(
        "Agent router: page content too large (est. {} tokens), max {} tokens; reducing",
        body_tokens_est, max_tokens_for_body
    );

    let body_truncated_for_request = truncate_at_boundary(body, max_chars);
    let summary_tokens = (max_tokens_for_body / 2).max(256);
    let summarization_messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: format!(
                "Summarize the following web page content in under {} tokens, keeping the most relevant information for answering questions. Output only the summary, no preamble.",
                summary_tokens
            ),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: body_truncated_for_request,
            images: None,
        },
    ];

    match send_ollama_chat_messages(summarization_messages, model_override, options_override).await
    {
        Ok(resp) => {
            let summary = resp.message.content.trim().to_string();
            if summary.is_empty() {
                let fallback = truncate_at_boundary(body, max_chars);
                Ok(format!(
                    "{} (content truncated due to context limit)",
                    fallback.trim_end()
                ))
            } else {
                Ok(summary)
            }
        }
        Err(e) => {
            info!("Agent router: summarization failed ({}), truncating", e);
            let fallback = truncate_at_boundary(body, max_chars);
            Ok(format!(
                "{} (content truncated due to context limit)",
                fallback.trim_end()
            ))
        }
    }
}

/// True if `needle` appears in `haystack` with no ASCII alnum / `_` immediately before or after.
/// Avoids treating `context_length_exceeded` as present inside `old_context_length_exceeded`.
fn contains_bounded_token(haystack: &str, needle: &str) -> bool {
    fn ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
    for (i, _) in haystack.match_indices(needle) {
        let before_ok = haystack[..i]
            .chars()
            .last()
            .is_none_or(|c| !ident_continue(c));
        let after_ok = haystack[i + needle.len()..]
            .chars()
            .next()
            .is_none_or(|c| !ident_continue(c));
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

/// True if `phrase` appears in `haystack` and is not immediately preceded by ASCII alnum / `_`
/// (so `rows exceed` does not match inside `arrows exceed` / `throws exceed`, `columns exceed`
/// does not match inside `microcolumns exceed`, `tables exceed` does not match inside
/// `constables exceed`, `table exceed` does not match inside `stable exceed`, `blocks exceed`
/// does not match inside `roadblocks exceed`, `block exceed` does not match inside
/// `roadblock exceed` / `sunblock exceed`, `segments exceed` does not match inside
/// `microsegments exceed`, `segment exceed` does not match inside `multisegment exceeds`,
/// `sections exceed` does not match inside `subsections exceed`, and `section exceed` does not
/// match inside `intersection exceed`, `paragraphs exceed` does not match inside
/// `counterparagraphs exceed`, and `paragraph exceed` does not match inside
/// `counterparagraph exceed`, `sentences exceed` does not match inside
/// `microsentences exceed`, and `sentence exceed` does not match inside
/// `microsentence exceed`, `words exceed` does not match inside
/// `buzzwords exceed` / `keywords exceed`, and `word exceed` does not match inside
/// `buzzword exceed`; `characters exceed` does not match inside `megacharacters exceed` /
/// `metacharacters exceed`, and `character exceed` does not match inside `noncharacter exceed`;
/// `bytes exceed` does not match inside `megabytes exceed` / `kilobytes exceed`, and
/// `byte exceed` does not match inside `kilobyte exceed`; `bits exceed` does not match inside
/// `megabits exceed` / `kilobits exceed`, and `bit exceed` does not match inside `kilobit exceed`
/// or as a substring of `rabbit exceed` (left-boundary rejects the inner `bit`); `fields exceed`
/// does not match inside `battlefields exceed` / `cornfields exceed`, and `field exceed` does not
/// match inside `afield exceed` / `subfield exceed`; `values exceed` does not match inside
/// `eigenvalues exceed` / `meanvalues exceed`, and `value exceed` does not match inside
/// `devalue exceed` / `overvalue exceed`; `keys exceed` does not match inside
/// `hotkeys exceed` / `turnkeys exceed`, and `key exceed` does not match inside
/// `monkey exceed` / `passkey exceed`; `properties exceed` does not match inside
/// `microproperties exceed`, and `property exceed` does not match inside
/// `subproperty exceed`; `schemas exceed` does not match inside `microschemas exceed` /
/// `holoschemas exceed`, and `schema exceed` does not match inside `subschema exceed`;
/// `parameters exceed` does not match inside `microparameters exceed` /
/// `metaparameters exceed`, and `parameter exceed` does not match inside
/// `subparameter exceed`; `arguments exceed` does not match inside
/// `microarguments exceed` / `metaarguments exceed`, and `argument exceed` does not match inside
/// `subargument exceed`; `variables exceed` does not match inside `metavariables exceed` /
/// `hypervariables exceed`, and `variable exceed` does not match inside `multivariable exceed` /
/// `subvariable exceed`; `headers exceed` does not match inside `microheaders exceed` /
/// `metaheaders exceed`, and `header exceed` does not match inside `subheader exceed`;
/// `cookies exceed` does not match inside `microcookies exceed` / `metacookies exceed`, and
/// `cookie exceed` does not match inside `subcookie exceed`; `bodies exceed` does not match
/// inside `microbodies exceed` / `metabodies exceed`, and `body exceed` does not match inside
/// `subbody exceed`; `parts exceed` does not match inside `microparts exceed` /
/// `metaparts exceed`, and `part exceed` does not match inside `subpart exceed`;
/// `pieces exceed` does not match inside `micropieces exceed` /
/// `metapieces exceed`, and `piece exceed` does not match inside `subpiece exceed`;
/// `shards exceed` does not match inside `microshards exceed` /
/// `metashards exceed`, and `shard exceed` does not match inside `subshard exceed`
/// (left-boundary rejects `reshard exceed`); `fragments exceed` does not match inside
/// `microfragments exceed` / `metafragments exceed`, and `fragment exceed` does not match inside
/// `subfragment exceed` (left-boundary rejects `refragment exceed`); `packets exceed` does not match
/// inside `micropackets exceed` / `metapackets exceed`, and `packet exceed` does not match inside
/// `subpacket exceed` (left-boundary rejects `repacket exceed`); `frames exceed` does not match
/// inside `microframes exceed` / `metaframes exceed`, and `frame exceed` does not match inside
/// `subframe exceed` (left-boundary rejects `reframe exceed`); `samples exceed` does not match
/// inside `microsamples exceed` / `metasamples exceed`, and `sample exceed` does not match inside
/// `subsample exceed` (left-boundary rejects `resample exceed`); `observations exceed` does not
/// match inside `microobservations exceed` / `metaobservations exceed`, and `observation exceed`
/// does not match inside `subobservation exceed` (left-boundary rejects `preobservation exceed`);
/// `events exceed` does not match inside `microevents exceed` / `metaevents exceed`, and
/// `event exceed` does not match inside `subevent exceed` (left-boundary rejects `preevent exceed`);
/// `traces exceed` does not match inside `microtraces exceed` / `metatraces exceed`, and
/// `trace exceed` does not match inside `subtrace exceed` (left-boundary rejects `pretrace exceed`
/// and `retrace exceed`);
/// `spans exceed` does not match inside `microspans exceed` / `metaspans exceed`, and
/// `span exceed` does not match inside `subspan exceed` (left-boundary rejects `prespan exceed`
/// and `respan exceed`);
/// `attributes exceed` does not match inside `microattributes exceed` / `metaattributes exceed`, and
/// `attribute exceed` does not match inside `subattribute exceed` (left-boundary rejects
/// `preattribute exceed` and `reattribute exceed`);
/// `links exceed` does not match inside `microlinks exceed` / `metalinks exceed`, and
/// `link exceed` does not match inside `sublink exceed` (left-boundary rejects `prelink exceed`
/// and `relink exceed`);
/// `scopes exceed` does not match inside `microscopes exceed` / `metascopes exceed`, and
/// `scope exceed` does not match inside `subscope exceed` (left-boundary rejects `prescope exceed`
/// and `rescope exceed`);
/// `resources exceed` does not match inside `microresources exceed` / `metaresources exceed`, and
/// `resource exceed` does not match inside `subresource exceed` (left-boundary rejects
/// `preresource exceed` and `reresource exceed`);
/// `metrics exceed` does not match inside `micrometrics exceed` / `metametrics exceed`, and
/// `metric exceed` does not match inside `submetric exceed` (left-boundary rejects
/// `premetric exceed` and `remetric exceed`);
/// `dimensions exceed` does not match inside `microdimensions exceed` / `metadimensions exceed`, and
/// `dimension exceed` does not match inside `subdimension exceed` (left-boundary rejects
/// `predimension exceed` and `redimension exceed`);
/// `tensors exceed` does not match inside `microtensors exceed` / `metatensors exceed`, and
/// `tensor exceed` does not match inside `subtensor exceed` (left-boundary rejects `pretensor exceed`
/// and `retensor exceed`);
/// `activations exceed` does not match inside `microactivations exceed` / `metaactivations exceed`, and
/// `activation exceed` does not match inside `subactivation exceed` (left-boundary rejects
/// `preactivation exceed` and `reactivation exceed`);
/// `gradients exceed` does not match inside `microgradients exceed` / `metagradients exceed`, and
/// `gradient exceed` does not match inside `subgradient exceed` (left-boundary rejects
/// `pregradient exceed` and `regradient exceed`);
/// `weights exceed` does not match inside `microweights exceed` / `metaweights exceed`, and
/// `weight exceed` does not match inside `subweight exceed` (left-boundary rejects `preweight exceed`
/// and `reweight exceed`);
/// `biases exceed` does not match inside `microbiases exceed` / `metabiases exceed`, and
/// `bias exceed` does not match inside `subbias exceed` (left-boundary rejects `prebias exceed`
/// and `rebias exceed`);
/// `layers exceed` does not match inside `microlayers exceed` / `metalayers exceed`, and
/// `layer exceed` does not match inside `sublayer exceed` (left-boundary rejects `prelayer exceed`
/// and `relayer exceed`);
/// `heads exceed` does not match inside `microheads exceed` / `metaheads exceed`, and
/// `head exceed` does not match inside `subhead exceed` (left-boundary rejects `prehead exceed`
/// and `rehead exceed`);
/// `positions exceed` does not match inside `micropositions exceed` / `metapositions exceed`, and
/// `position exceed` does not match inside `subposition exceed` (left-boundary rejects
/// `preposition exceed` and `reposition exceed`);
/// `embeddings exceed` does not match inside `microembeddings exceed` / `metaembeddings exceed`, and
/// `embedding exceed` does not match inside `subembedding exceed` (left-boundary rejects
/// `preembedding exceed` and `reembedding exceed`);
/// `logits exceed` does not match inside `micrologits exceed` / `metalogits exceed`, and
/// `logit exceed` does not match inside `sublogit exceed` (left-boundary rejects `prelogit exceed`
/// and `relogit exceed`);
/// `probabilities exceed` does not match inside `microprobabilities exceed` /
/// `metaprobabilities exceed`, and `probability exceed` does not match inside `subprobability exceed`
/// (left-boundary rejects `preprobability exceed` and `reprobability exceed`);
/// `logprobs exceed` does not match inside `micrologprobs exceed` / `metalogprobs exceed`, and
/// `logprob exceed` does not match inside `sublogprob exceed` (left-boundary rejects `prelogprob exceed`
/// and `relogprob exceed`);
/// `arrays exceed` does not match inside `microarrays exceed` / `metaarrays exceed`, and
/// `array exceed` does not match inside `subarray exceed` (left-boundary rejects `prearray exceed`
/// and `rearray exceed`);
/// `objects exceed` does not match inside `microobjects exceed` / `metaobjects exceed`, and
/// `object exceed` does not match inside `subobject exceed` (left-boundary rejects `preobject exceed`
/// and `reobject exceed`);
/// `elements exceed` does not match inside `microelements exceed` / `metaelements exceed`, and
/// `element exceed` does not match inside `subelement exceed` (left-boundary rejects `preelement exceed`
/// and `reelement exceed`);
/// `nodes exceed` does not match inside `micronodes exceed` / `metanodes exceed`, and
/// `node exceed` does not match inside `subnode exceed` (left-boundary rejects `prenode exceed`
/// and `renode exceed`);
/// `edges exceed` does not match inside `microedges exceed` / `metaedges exceed`, and
/// `edge exceed` does not match inside `subedge exceed` (left-boundary rejects `preedge exceed`
/// and `reedge exceed`);
/// `vertices exceed` does not match inside `microvertices exceed` / `metavertices exceed`, and
/// `vertex exceed` does not match inside `subvertex exceed` (left-boundary rejects `prevertex exceed`
/// and `revertex exceed`);
/// `faces exceed` does not match inside `microfaces exceed` / `metafaces exceed`, and
/// `face exceed` does not match inside `subface exceed` (left-boundary rejects `preface exceed`
/// and `reface exceed`);
/// `triangles exceed` does not match inside `microtriangles exceed` / `metatriangles exceed`, and
/// `triangle exceed` does not match inside `subtriangle exceed` (left-boundary rejects
/// `pretriangle exceed` and `retriangle exceed`);
/// `polygons exceed` does not match inside `micropolygons exceed` / `metapolygons exceed`, and
/// `polygon exceed` does not match inside `subpolygon exceed` (left-boundary rejects
/// `prepolygon exceed` and `repolygon exceed`);
/// `meshes exceed` does not match inside `micromeshes exceed` / `metameshes exceed`, and
/// `mesh exceed` does not match inside `submesh exceed` (left-boundary rejects `premesh exceed`
/// and `remesh exceed`);
/// `voxels exceed` does not match inside `microvoxels exceed` / `metavoxels exceed`, and
/// `voxel exceed` does not match inside `subvoxel exceed` (left-boundary rejects `prevoxel exceed`
/// and `revoxel exceed`);
/// `particles exceed` does not match inside `microparticles exceed` / `metaparticles exceed`, and
/// `particle exceed` does not match inside `subparticle exceed` (left-boundary rejects `preparticle exceed`
/// and `reparticle exceed`);
/// `molecules exceed` does not match inside `micromolecules exceed` / `metamolecules exceed`, and
/// `molecule exceed` does not match inside `submolecule exceed` (left-boundary rejects `premolecule exceed`
/// and `remolecule exceed`);
/// `atoms exceed` does not match inside `microatoms exceed` / `metaatoms exceed`, and
/// `atom exceed` does not match inside `subatom exceed` (left-boundary rejects `preatom exceed`
/// and `reatom exceed`);
/// `ions exceed` does not match inside `microions exceed` / `metaions exceed`, and
/// `ion exceed` does not match inside `subion exceed` (left-boundary rejects `preion exceed`
/// and `reion exceed`);
/// `electrons exceed` does not match inside `microelectrons exceed` / `metaelectrons exceed`, and
/// `electron exceed` does not match inside `subelectron exceed` (left-boundary rejects
/// `preelectron exceed` and `reelectron exceed`);
/// `protons exceed` does not match inside `microprotons exceed` / `metaprotons exceed`, and
/// `proton exceed` does not match inside `subproton exceed` (left-boundary rejects
/// `preproton exceed` and `reproton exceed`);
/// `neutrons exceed` does not match inside `microneutrons exceed` / `metaneutrons exceed`, and
/// `neutron exceed` does not match inside `subneutron exceed` (left-boundary rejects
/// `preneutron exceed` and `reneutron exceed`);
/// `quarks exceed` does not match inside `microquarks exceed` / `metaquarks exceed`, and
/// `quark exceed` does not match inside `subquark exceed` (left-boundary rejects
/// `prequark exceed` and `requark exceed`);
/// `gluons exceed` does not match inside `microgluons exceed` / `metagluons exceed`, and
/// `gluon exceed` does not match inside `subgluon exceed` (left-boundary rejects
/// `pregluon exceed` and `regluon exceed`);
/// `bosons exceed` does not match inside `microbosons exceed` / `metabosons exceed`, and
/// `boson exceed` does not match inside `subboson exceed` (left-boundary rejects
/// `preboson exceed` and `reboson exceed`);
/// `leptons exceed` does not match inside `microleptons exceed` / `metaleptons exceed`, and
/// `lepton exceed` does not match inside `sublepton exceed` (left-boundary rejects
/// `prelepton exceed` and `relepton exceed`);
/// `hadrons exceed` does not match inside `microhadrons exceed` / `metahadrons exceed`, and
/// `hadron exceed` does not match inside `subhadron exceed` (left-boundary rejects
/// `prehadron exceed` and `rehadron exceed`);
/// `photons exceed` does not match inside `microphotons exceed` / `metaphotons exceed`, and
/// `photon exceed` does not match inside `subphoton exceed` (left-boundary rejects
/// `prephoton exceed` and `rephoton exceed`);
/// `phonons exceed` does not match inside `microphonons exceed` / `metaphonons exceed`, and
/// `phonon exceed` does not match inside `subphonon exceed` (left-boundary rejects
/// `prephonon exceed` and `rephonon exceed`);
/// `excitons exceed` does not match inside `microexcitons exceed` / `metaexcitons exceed`, and
/// `exciton exceed` does not match inside `subexciton exceed` (left-boundary rejects
/// `preexciton exceed` and `reexciton exceed`);
/// `polarons exceed` does not match inside `micropolarons exceed` / `metapolarons exceed`, and
/// `polaron exceed` does not match inside `subpolaron exceed` (left-boundary rejects
/// `prepolaron exceed` and `repolaron exceed`);
/// `plasmons exceed` does not match inside `microplasmons exceed` / `metaplasmons exceed`, and
/// `plasmon exceed` does not match inside `subplasmon exceed` (left-boundary rejects
/// `preplasmon exceed` and `replasmon exceed`);
/// `solitons exceed` does not match inside `microsolitons exceed` / `metasolitons exceed`, and
/// `soliton exceed` does not match inside `subsoliton exceed` (left-boundary rejects
/// `presoliton exceed` and `resoliton exceed`);
/// `instantons exceed` does not match inside `microinstantons exceed` / `metainstantons exceed`, and
/// `instanton exceed` does not match inside `subinstanton exceed` (left-boundary rejects
/// `preinstanton exceed` and `reinstanton exceed`);
/// `skyrmions exceed` does not match inside `microskyrmions exceed` / `metaskyrmions exceed`, and
/// `skyrmion exceed` does not match inside `subskyrmion exceed` (left-boundary rejects
/// `preskyrmion exceed` and `reskyrmion exceed`);
/// `magnons exceed` does not match inside `micromagnons exceed` / `metamagnons exceed`, and
/// `magnon exceed` does not match inside `submagnon exceed` (left-boundary rejects
/// `premagnon exceed` and `remagnon exceed`);
/// `rotons exceed` does not match inside `microrotons exceed` / `metarotons exceed`, and
/// `roton exceed` does not match inside `subroton exceed` (left-boundary rejects
/// `preroton exceed` and `reroton exceed`);
/// `anyons exceed` does not match inside `microanyons exceed` / `metaanyons exceed`, and
/// `anyon exceed` does not match inside `subanyon exceed` (left-boundary rejects
/// `preanyon exceed` and `reanyon exceed`);
/// `fluxons exceed` does not match inside `microfluxons exceed` / `metafluxons exceed`, and
/// `fluxon exceed` does not match inside `subfluxon exceed` (left-boundary rejects
/// `prefluxon exceed` and `refluxon exceed`);
/// `vortices exceed` does not match inside `microvortices exceed` / `metavortices exceed`, and
/// `vortex exceed` does not match inside `subvortex exceed` (left-boundary rejects
/// `prevortex exceed` and `revortex exceed`);
/// `messages exceed` does not match inside `micromessages exceed` / `metamessages exceed`, and
/// `message exceed` does not match inside `submessage exceed` (left-boundary rejects `premessage exceed`
/// and `remessage exceed`);
/// `inputs exceed` does not match inside `microinputs exceed` / `metainputs exceed`, and
/// `input exceed` does not match inside `subinput exceed` (left-boundary rejects `preinput exceed`
/// and `reinput exceed`);
/// `contents exceed` does not match inside `microcontents exceed` / `metacontents exceed`, and
/// `content exceed` does not match inside `subcontent exceed` (left-boundary rejects `precontent exceed`
/// and `recontent exceed`);
/// `outputs exceed` does not match inside `microoutputs exceed` / `metaoutputs exceed`, and
/// `output exceed` does not match inside `suboutput exceed` (left-boundary rejects `preoutput exceed`
/// and `reoutput exceed`);
/// `responses exceed` does not match inside `microresponses exceed` / `metaresponses exceed`, and
/// `response exceed` does not match inside `subresponse exceed` (left-boundary rejects `preresponse exceed`
/// and `reresponse exceed`);
/// `requests exceed` does not match inside `microrequests exceed` / `metarequests exceed`, and
/// `request exceed` does not match inside `subrequest exceed` (left-boundary rejects `prerequest exceed`);
/// `queries exceed` does not match inside `microqueries exceed` / `metaqueries exceed`, and
/// `query exceed` does not match inside `subquery exceed` (left-boundary rejects `prequery exceed` and `requery exceed`);
/// `records exceed` does not match inside `microrecords exceed` / `metarecords exceed`, and
/// `record exceed` does not match inside `subrecord exceed` (left-boundary rejects `rerecord exceed`
/// and `prerecord exceed`);
/// `total prompt tokens exceed` does not match inside `micrototal prompt tokens exceed`;
/// `requested tokens exceed` does not match inside `microrequested tokens exceed`;
/// `total tokens exceed` does not match inside `micrototal tokens exceed` / `metatotal tokens exceed`;
/// `prompt tokens exceed` does not match inside `subprompt tokens exceed` / `preprompt tokens exceed`,
/// nor when the preceding ident token is a compound ending in `total` (e.g. `micrototal prompt tokens exceed`);
/// `input tokens exceed` does not match inside `subinput tokens exceed` / `preinput tokens exceed`;
/// plural `tokens exceed` / `tokens exceeded` skip matches whose preceding token is `prompt`, `input`,
/// or `total` (so `micrototal prompt tokens exceed` is not matched on the `tokens exceed` arm);
/// `sequence length exceeds` does not match inside `subsequence length exceeds`;
/// `exceeds maximum sequence length` does not match inside `microexceeds maximum sequence length`;
/// `maximum sequence length exceeded` does not match inside `micromaximum sequence length exceeded`;
/// `input sequence is too long` does not match inside `microinput sequence is too long`;
/// `prompt length exceeds` does not match inside `microprompt length exceeds`;
/// `prompt too long` / `prompt is too long` do not match inside `microprompt too long` / `microprompt is too long`;
/// `prompt has more tokens than` does not match inside `microprompt has more tokens than`;
/// `prompt exceeds the context` does not match inside `microprompt exceeds the context`;
/// `context overflow` does not match inside `microcontext overflow`;
/// `context length exceeded` does not match inside `microcontext length exceeded`;
/// `maximum context length` does not match inside `micromaximum context length`;
/// `exceeds the model's context window` / `exceeds the context window` / `context window exceeded` /
/// `exceeded the context limit` (and parallel `model context` / `model's maximum context` phrases)
/// do not match when the same substrings are embedded after an ASCII ident continuation
/// (e.g. `microexceeds the model's context window`, `micromodel context exceeded`);
/// `requested more tokens than` / `fit in the context` / `larger than the context` /
/// preposition + `context window` phrases (`outside` / `beyond` / `over`, with optional `the`) /
/// `beyond the context`, and `ran out of context` / `running out of context` do not match when embedded after ident
/// continuation (e.g. `microrequested more tokens than`, `microfit in the context window`, `microbeyond the context window`);
/// fixed prose phrases such as `context size exceeded`, `exceeds available context`, `past the context`,
/// `context exhausted`, `exceed the maximum context`, context token limit wording, and `exceeds this model's context`
/// likewise do not match when embedded after ident continuation (FEAT-D387). Compound `… && …` arms for
/// `context budget`, `conversation` + `too long` + `context`, `too long`/`too large`/fit phrasing + `context`,
/// `context buffer`/`configured context`/`context capacity`/`allocated context`, `truncated` + context slots,
/// `kv cache`/`prefill` + `context`, etc. use the same left-boundary rule so `microcontext` alone does not
/// satisfy a bare `context` conjunct (FEAT-D388). The FEAT-D295 explicit context-slot OR list
/// (`context window`, `max context`, `model's context`, etc.) and the loose `total tokens exceed` /
/// token-arm slot disjuncts use the same boundary rule so `microcontext window` does not satisfy the slot
/// (FEAT-D389). Bounded JSON-key arms use ident-boundary on overflow verbs (`exceed`, `overflow`, …)
/// so `microexceed` inside a compound does not satisfy the second conjunct; word-order arms for
/// `maximum context` / `max context` / `maximum allowed context` / `context length limit` and
/// FEAT-D309–D319 `calls`/`batches`/…/`cells` exceed phrases use the same rule (FEAT-D390).
/// The FEAT-D295-guarded `message`/`messages`/`input`/`inputs` … `too long` phrases use the same
/// boundary rule so that conjunct does not use plain substring `contains` on those phrases (FEAT-D391).
fn contains_phrase_after_ident_boundary(haystack: &str, phrase: &str) -> bool {
    fn ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
    for (i, _) in haystack.match_indices(phrase) {
        if i == 0 {
            return true;
        }
        if !haystack[..i]
            .chars()
            .next_back()
            .is_some_and(ident_continue)
        {
            return true;
        }
    }
    false
}

/// ASCII `[a-zA-Z0-9_]+` token immediately before `phrase_start` (skipping whitespace).
fn preceding_ascii_ident_token(haystack: &str, phrase_start: usize) -> Option<&str> {
    if phrase_start == 0 {
        return None;
    }
    let b = haystack.as_bytes();
    let mut end = phrase_start;
    while end > 0 && b[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    if end == 0 {
        return None;
    }
    let mut start = end;
    while start > 0 {
        let c = b[start - 1];
        if c.is_ascii_alphanumeric() || c == b'_' {
            start -= 1;
        } else {
            break;
        }
    }
    if start == end {
        None
    } else {
        Some(&haystack[start..end])
    }
}

/// `prompt tokens exceed` with ident-boundary, skipping matches where the prior token is `*total` (not bare `total`).
fn contains_prompt_tokens_exceed_after_boundary_excluding_compound_total(haystack: &str) -> bool {
    const PHRASE: &str = "prompt tokens exceed";
    fn ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
    for (i, _) in haystack.match_indices(PHRASE) {
        if i > 0 && haystack[..i].chars().next_back().is_some_and(ident_continue) {
            continue;
        }
        if preceding_ascii_ident_token(haystack, i).is_some_and(|t| t != "total" && t.ends_with("total"))
        {
            continue;
        }
        return true;
    }
    false
}

/// `tokens exceed` / `tokens exceeded` with ident-boundary; skips when the prior token is `prompt` / `input` /
/// `total` / `requested` or a longer ASCII ident ending with one of those roots (e.g. `microrequested`).
fn contains_tokens_exceed_subphrase_at_boundary(haystack: &str, phrase: &str) -> bool {
    fn ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }
    fn skip_prev_token_for_tokens_arm(t: &str) -> bool {
        ["prompt", "input", "total", "requested"]
            .iter()
            .any(|root| t.ends_with(*root))
    }
    for (i, _) in haystack.match_indices(phrase) {
        if i > 0 && haystack[..i].chars().next_back().is_some_and(ident_continue) {
            continue;
        }
        if preceding_ascii_ident_token(haystack, i).is_some_and(skip_prev_token_for_tokens_arm) {
            continue;
        }
        return true;
    }
    false
}

/// Explicit context-slot phrases for FEAT-D295-style guards (not bare `model`).
/// Ident-boundary on each phrase so `microcontext window` / `micromaximum context` do not satisfy
/// the slot (FEAT-D389).
fn explicit_context_slot_after_ident_boundary(lower: &str) -> bool {
    contains_phrase_after_ident_boundary(lower, "context window")
        || contains_phrase_after_ident_boundary(lower, "context length")
        || contains_phrase_after_ident_boundary(lower, "context limit")
        || contains_phrase_after_ident_boundary(lower, "context size")
        || contains_phrase_after_ident_boundary(lower, "max context")
        || contains_phrase_after_ident_boundary(lower, "maximum context")
        || contains_phrase_after_ident_boundary(lower, "available context")
        || contains_phrase_after_ident_boundary(lower, "model's context")
}

/// Loose `context` / model-related slot under `total tokens exceed` / token arms (FEAT-D389).
fn token_overflow_slot_conjunct_after_ident_boundary(lower: &str, include_window: bool) -> bool {
    let base = contains_phrase_after_ident_boundary(lower, "context")
        || contains_phrase_after_ident_boundary(lower, "model")
        || contains_phrase_after_ident_boundary(lower, "maximum")
        || contains_phrase_after_ident_boundary(lower, "sequence")
        || contains_phrase_after_ident_boundary(lower, "n_ctx");
    if include_window {
        base || contains_phrase_after_ident_boundary(lower, "window")
    } else {
        base
    }
}

/// Check whether an Ollama error string indicates a context-window overflow.
pub(crate) fn is_context_overflow_error(err: &str) -> bool {
    let lower = err.to_lowercase();
    contains_phrase_after_ident_boundary(&lower, "context overflow")
        || contains_phrase_after_ident_boundary(&lower, "prompt too long")
        || contains_phrase_after_ident_boundary(&lower, "prompt is too long")
        || contains_phrase_after_ident_boundary(&lower, "context length exceeded")
        || contains_phrase_after_ident_boundary(&lower, "maximum context length")
        || contains_phrase_after_ident_boundary(&lower, "exceeds the model's context window")
        || contains_phrase_after_ident_boundary(&lower, "exceeds the model context")
        || contains_phrase_after_ident_boundary(&lower, "exceeded the model context")
        || contains_phrase_after_ident_boundary(&lower, "exceed the model context")
        || contains_phrase_after_ident_boundary(&lower, "model context exceeded")
        || contains_phrase_after_ident_boundary(&lower, "model context limit exceeded")
        || contains_phrase_after_ident_boundary(&lower, "exceeds the model's maximum context")
        || contains_phrase_after_ident_boundary(&lower, "exceeded the model's maximum context")
        || contains_phrase_after_ident_boundary(&lower, "exceed the model's maximum context")
        || contains_phrase_after_ident_boundary(&lower, "exceeds the context window")
        || contains_phrase_after_ident_boundary(&lower, "context window exceeded")
        || contains_phrase_after_ident_boundary(&lower, "exceeded the context limit")
        || contains_phrase_after_ident_boundary(&lower, "exceeds the context limit")
        || contains_phrase_after_ident_boundary(&lower, "exceed the context limit")
        || contains_phrase_after_ident_boundary(&lower, "requested more tokens than")
        // Ident-boundary (FEAT-D377): `micrototal prompt tokens exceed` does not embed
        // `total prompt tokens exceed` at a boundary; parallel to `inputs exceed` (FEAT-D375).
        || contains_phrase_after_ident_boundary(&lower, "total prompt tokens exceed")
        || contains_phrase_after_ident_boundary(&lower, "fit in the context")
        || contains_phrase_after_ident_boundary(&lower, "larger than the context")
        || contains_phrase_after_ident_boundary(&lower, "outside the context window")
        || contains_phrase_after_ident_boundary(&lower, "outside of the context window")
        || contains_phrase_after_ident_boundary(&lower, "beyond the context window")
        || contains_phrase_after_ident_boundary(&lower, "over the context window")
        // Compact wording: no `the` between preposition and `context window` (FEAT-D304;
        // distinct from FEAT-D291's `beyond the` / `over the` / `outside of the` substrings).
        || contains_phrase_after_ident_boundary(&lower, "beyond context window")
        || contains_phrase_after_ident_boundary(&lower, "over context window")
        || contains_phrase_after_ident_boundary(&lower, "outside context window")
        || contains_phrase_after_ident_boundary(&lower, "outside of context window")
        || contains_phrase_after_ident_boundary(&lower, "ran out of context")
        || contains_phrase_after_ident_boundary(&lower, "running out of context")
        || (contains_phrase_after_ident_boundary(&lower, "context budget")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "full")))
        || (contains_phrase_after_ident_boundary(&lower, "conversation")
            && contains_phrase_after_ident_boundary(&lower, "too long")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || contains_phrase_after_ident_boundary(&lower, "context size exceeded")
        || contains_phrase_after_ident_boundary(&lower, "exceeded context size")
        || contains_phrase_after_ident_boundary(&lower, "prompt has more tokens than")
        || contains_phrase_after_ident_boundary(&lower, "exceeds available context")
        || contains_phrase_after_ident_boundary(&lower, "context limit exceeded")
        || contains_phrase_after_ident_boundary(&lower, "exceeds context length")
        || contains_phrase_after_ident_boundary(&lower, "requested tokens exceed")
        || (contains_phrase_after_ident_boundary(&lower, "too long")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || (contains_phrase_after_ident_boundary(&lower, "too large")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || (contains_phrase_after_ident_boundary(&lower, "cannot fit")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || (contains_phrase_after_ident_boundary(&lower, "does not fit")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || (contains_phrase_after_ident_boundary(&lower, "doesn't fit")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || (contains_phrase_after_ident_boundary(&lower, "unable to fit")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || (contains_phrase_after_ident_boundary(&lower, "won't fit")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || (contains_phrase_after_ident_boundary(&lower, "longer than")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || contains_phrase_after_ident_boundary(&lower, "exceeds maximum context")
        || contains_phrase_after_ident_boundary(&lower, "maximum context exceeded")
        || contains_phrase_after_ident_boundary(&lower, "insufficient context")
        || contains_phrase_after_ident_boundary(&lower, "prompt exceeds the context")
        || (contains_phrase_after_ident_boundary(&lower, "greater than")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || (contains_bounded_token(&lower, "n_ctx")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too small")))
        || (contains_bounded_token(&lower, "num_ctx")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "larger")
                || contains_phrase_after_ident_boundary(&lower, "greater")
                || contains_phrase_after_ident_boundary(&lower, "longer")))
        || contains_phrase_after_ident_boundary(&lower, "exceeds maximum sequence length")
        || contains_phrase_after_ident_boundary(&lower, "maximum sequence length exceeded")
        || contains_phrase_after_ident_boundary(&lower, "sequence length exceeds")
        || contains_phrase_after_ident_boundary(&lower, "input sequence is too long")
        || (contains_phrase_after_ident_boundary(&lower, "total tokens exceed")
            && token_overflow_slot_conjunct_after_ident_boundary(&lower, false))
        || contains_phrase_after_ident_boundary(&lower, "prompt length exceeds")
        || (contains_phrase_after_ident_boundary(&lower, "too many tokens")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || contains_phrase_after_ident_boundary(&lower, "max context exceeded")
        || contains_phrase_after_ident_boundary(&lower, "exceeds max context")
        || contains_phrase_after_ident_boundary(&lower, "beyond the context")
        || contains_phrase_after_ident_boundary(&lower, "not enough context")
        || (contains_phrase_after_ident_boundary(&lower, "context buffer")
            && (contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "full")))
        || ((contains_prompt_tokens_exceed_after_boundary_excluding_compound_total(&lower)
            || contains_phrase_after_ident_boundary(&lower, "input tokens exceed"))
            && token_overflow_slot_conjunct_after_ident_boundary(&lower, true))
        || contains_phrase_after_ident_boundary(&lower, "exceeds the configured context")
        || (contains_phrase_after_ident_boundary(&lower, "configured context")
            && (contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "full")))
        || (contains_phrase_after_ident_boundary(&lower, "would exceed")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || contains_phrase_after_ident_boundary(&lower, "past the context")
        || contains_phrase_after_ident_boundary(&lower, "reached the context limit")
        || contains_phrase_after_ident_boundary(&lower, "hit the context limit")
        || contains_phrase_after_ident_boundary(&lower, "over the context limit")
        || (contains_phrase_after_ident_boundary(&lower, "truncated")
            && (contains_phrase_after_ident_boundary(&lower, "context window")
                || contains_phrase_after_ident_boundary(&lower, "context length")
                || contains_phrase_after_ident_boundary(&lower, "context limit")
                || contains_phrase_after_ident_boundary(&lower, "max context")))
        || contains_phrase_after_ident_boundary(&lower, "context exhausted")
        || (contains_phrase_after_ident_boundary(&lower, "context")
            && (contains_phrase_after_ident_boundary(&lower, "fully exhausted")
                || contains_phrase_after_ident_boundary(&lower, "completely exhausted")))
        || contains_phrase_after_ident_boundary(&lower, "insufficient remaining context")
        || ((contains_phrase_after_ident_boundary(&lower, "kv cache")
            || contains_phrase_after_ident_boundary(&lower, "kv-cache"))
            && contains_phrase_after_ident_boundary(&lower, "is full")
            && contains_phrase_after_ident_boundary(&lower, "context"))
        || contains_phrase_after_ident_boundary(&lower, "exceeds the context size")
        || contains_phrase_after_ident_boundary(&lower, "context size exceeds")
        || contains_phrase_after_ident_boundary(&lower, "exceeded the context size")
        || (contains_phrase_after_ident_boundary(&lower, "context window")
            && contains_phrase_after_ident_boundary(&lower, "too small"))
        || (contains_phrase_after_ident_boundary(&lower, "context capacity")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "full")))
        || (contains_phrase_after_ident_boundary(&lower, "allocated context")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "full")))
        || (contains_phrase_after_ident_boundary(&lower, "prefill")
            && contains_phrase_after_ident_boundary(&lower, "context")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")))
        || (contains_bounded_token(&lower, "max_context")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")
                || contains_phrase_after_ident_boundary(&lower, "larger")
                || contains_phrase_after_ident_boundary(&lower, "greater")))
        || (contains_bounded_token(&lower, "context_length")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")))
        // camelCase JSON / gateway errors (to_lowercase → "maxcontext" / "contextlength")
        || (contains_bounded_token(&lower, "maxcontext")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")
                || contains_phrase_after_ident_boundary(&lower, "larger")
                || contains_phrase_after_ident_boundary(&lower, "greater")))
        || (contains_bounded_token(&lower, "contextlength")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")))
        || (contains_bounded_token(&lower, "n_ctx_per_seq")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")
                || contains_phrase_after_ident_boundary(&lower, "larger")
                || contains_phrase_after_ident_boundary(&lower, "greater")
                || contains_phrase_after_ident_boundary(&lower, "too small")))
        // snake_case JSON / config (distinct from prose "context window")
        || (contains_bounded_token(&lower, "context_window")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")
                || contains_phrase_after_ident_boundary(&lower, "larger")
                || contains_phrase_after_ident_boundary(&lower, "greater")
                || contains_phrase_after_ident_boundary(&lower, "too small")))
        || (contains_bounded_token(&lower, "context_limit")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")))
        || (contains_bounded_token(&lower, "contextlimit")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")))
        || contains_phrase_after_ident_boundary(&lower, "exceed the maximum context")
        || contains_phrase_after_ident_boundary(&lower, "exceeded maximum context")
        || contains_phrase_after_ident_boundary(&lower, "maximum context is exceeded")
        || contains_phrase_after_ident_boundary(&lower, "context token limit exceeded")
        || contains_phrase_after_ident_boundary(&lower, "exceeds the context token limit")
        || contains_phrase_after_ident_boundary(&lower, "exceeded the context token limit")
        // Word order / filler between "exceed" and "maximum … context" (e.g. "model", "allowed")
        || (contains_phrase_after_ident_boundary(&lower, "maximum context")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")
                || contains_phrase_after_ident_boundary(&lower, "larger")
                || contains_phrase_after_ident_boundary(&lower, "greater")))
        || (contains_phrase_after_ident_boundary(&lower, "max context")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")
                || contains_phrase_after_ident_boundary(&lower, "larger")
                || contains_phrase_after_ident_boundary(&lower, "greater")))
        // "allowed" between max/imum and context breaks contiguous `maximum context` / `max context` substrings.
        || (contains_phrase_after_ident_boundary(&lower, "maximum allowed context")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")
                || contains_phrase_after_ident_boundary(&lower, "larger")
                || contains_phrase_after_ident_boundary(&lower, "greater")))
        || (contains_phrase_after_ident_boundary(&lower, "max allowed context")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")
                || contains_phrase_after_ident_boundary(&lower, "larger")
                || contains_phrase_after_ident_boundary(&lower, "greater")))
        || (contains_phrase_after_ident_boundary(&lower, "context length limit")
            && (contains_phrase_after_ident_boundary(&lower, "exceed")
                || contains_phrase_after_ident_boundary(&lower, "overflow")
                || contains_phrase_after_ident_boundary(&lower, "too long")
                || contains_phrase_after_ident_boundary(&lower, "too large")
                || contains_phrase_after_ident_boundary(&lower, "larger")
                || contains_phrase_after_ident_boundary(&lower, "greater")))
        // OpenAI-style / JSON `code` values and similar (bounded so `old_context_length_exceeded` etc. do not match)
        || contains_bounded_token(&lower, "context_budget_exceeded")
        || contains_bounded_token(&lower, "context_length_exceeded")
        || contains_bounded_token(&lower, "max_context_length_exceeded")
        || contains_bounded_token(&lower, "context_window_exceeded")
        || contains_bounded_token(&lower, "max_context_exceeded")
        // Demonstrative phrasing (distinct from `the model's` already covered above)
        || contains_phrase_after_ident_boundary(&lower, "exceeds this model's context")
        || contains_phrase_after_ident_boundary(&lower, "exceeded this model's context")
        || contains_phrase_after_ident_boundary(&lower, "exceed this model's context")
        // Chat-completions style: "messages exceed …" without "total tokens" wording.
        // Require explicit context-slot phrases (not bare `model`) so lines like
        // "status messages exceed limits (no model context)" do not match.
        // Possessive `model's context` is a slot (e.g. "too long for this model's context") and
        // does not substring-match bare `model context` in "(no model context configured)".
        // Ident-boundary (FEAT-D358): `micromessages exceed` / `metamessages exceed` do not embed
        // `messages exceed` at a boundary; parallel to `requests exceed` (FEAT-D353).
        || ((contains_phrase_after_ident_boundary(&lower, "messages exceed")
            || contains_phrase_after_ident_boundary(&lower, "messages exceeded"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Singular "message exceed(s/ed)" (parallel to plural `messages exceed`, FEAT-D298).
        // `message exceed` matches present/past via `exceed` prefix of `exceeds` / `exceeded`.
        // Does not substring-match plural `messages exceed` (the `s` after `message` breaks
        // `message` + space + `exceed`). Ident-boundary (FEAT-D358): `submessage exceed` /
        // `micromessage exceeds` do not false-positive.
        || (contains_phrase_after_ident_boundary(&lower, "message exceed")
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural "inputs exceed …" (present / past). Wording like `inputs exceed the context window`
        // does not contain the substring `exceeds the context window` (bare `exceed` before the slot).
        // Same context-slot guard as `messages exceed` (FEAT-D295).
        // Ident-boundary (FEAT-D375): `microinputs exceed` / `metainputs exceed` do not embed
        // `inputs exceed` at a boundary; parallel to `messages exceed` (FEAT-D358).
        || ((contains_phrase_after_ident_boundary(&lower, "inputs exceed")
            || contains_phrase_after_ident_boundary(&lower, "inputs exceeded"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Singular "input exceed(s/ed)" (parallel to plural `inputs exceed`, FEAT-D300).
        // `input exceed` matches present/past via `exceed` prefix of `exceeds` / `exceeded`.
        // Does not substring-match `inputs exceed` (letter `s` between `input` and `exceed`).
        // Ident-boundary (FEAT-D375): `subinput exceed` / `preinput exceed` / `reinput exceed` likewise.
        || (contains_phrase_after_ident_boundary(&lower, "input exceed")
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "content(s) exceed(s/ed)" (FEAT-D303; ident-boundary FEAT-D379). Parallel
        // to `inputs exceed` / `outputs exceed`. `content exceed` matches present/past via `exceed`
        // prefix of `exceeds` / `exceeded` and does not substring-match plural `contents exceed` (the
        // `s` after `content` breaks `content` + space + `exceed`). Ident-boundary so
        // `microcontents exceed` / `metacontents exceed` / `subcontent exceed` do not false-positive;
        // `precontent exceed` and `recontent exceed` are rejected the same way. Same context-slot
        // guard as `inputs exceed`.
        || ((contains_phrase_after_ident_boundary(&lower, "contents exceed")
            || contains_phrase_after_ident_boundary(&lower, "contents exceeded")
            || contains_phrase_after_ident_boundary(&lower, "content exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "output(s) exceed(s/ed)" (FEAT-D305; ident-boundary FEAT-D376). Parallel
        // to `inputs exceed` / `content exceed`. `output exceed` matches present/past via `exceed`
        // prefix of `exceeds` / `exceeded` and does not substring-match `outputs exceed` (the `s`
        // after `output`). Ident-boundary so `microoutputs exceed` / `metaoutputs exceed` /
        // `suboutput exceed` do not false-positive; `preoutput exceed` and `reoutput exceed` are
        // rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "outputs exceed")
            || contains_phrase_after_ident_boundary(&lower, "outputs exceeded")
            || contains_phrase_after_ident_boundary(&lower, "output exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "response(s) exceed(s/ed)" (FEAT-D306; ident-boundary FEAT-D378). Parallel
        // to `outputs exceed` / `inputs exceed`. `response exceed` matches present/past via `exceed`
        // prefix of `exceeds` / `exceeded` and does not substring-match `responses exceed` (the `s`
        // after `response`). Ident-boundary so `microresponses exceed` / `metaresponses exceed` /
        // `subresponse exceed` do not false-positive; `preresponse exceed` and `reresponse exceed` are
        // rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "responses exceed")
            || contains_phrase_after_ident_boundary(&lower, "responses exceeded")
            || contains_phrase_after_ident_boundary(&lower, "response exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "request(s) exceed(s/ed)" (FEAT-D307; ident-boundary FEAT-D353). Parallel
        // to `responses exceed` / `inputs exceed`. `request exceed` matches present/past via `exceed`
        // prefix of `exceeds` / `exceeded` and does not substring-match `requests exceed` (the `s`
        // after `request`). Ident-boundary so `microrequests exceed` / `metarequests exceed` /
        // `subrequest exceed` do not false-positive; `prerequest exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "requests exceed")
            || contains_phrase_after_ident_boundary(&lower, "requests exceeded")
            || contains_phrase_after_ident_boundary(&lower, "request exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "quer(y/ies) exceed(s/ed)" (FEAT-D308; ident-boundary FEAT-D380). Parallel
        // to `requests exceed` / `inputs exceed`. `query exceed` matches present/past via `exceed`
        // prefix of `exceeds` / `exceeded` and does not substring-match plural `queries exceed` (no
        // `query` + ` exceed` inside the word `queries`). Ident-boundary so `microqueries exceed` /
        // `metaqueries exceed` / `subquery exceed` do not false-positive; `prequery exceed` and
        // `requery exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "queries exceed")
            || contains_phrase_after_ident_boundary(&lower, "queries exceeded")
            || contains_phrase_after_ident_boundary(&lower, "query exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "call(s) exceed(s/ed)" (FEAT-D309; ident-boundary FEAT-D390). Parallel to `queries exceed` /
        // `requests exceed`. `call exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `calls exceed` (the `s` after `call`).
        || ((contains_phrase_after_ident_boundary(&lower, "calls exceed")
            || contains_phrase_after_ident_boundary(&lower, "calls exceeded")
            || contains_phrase_after_ident_boundary(&lower, "call exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "batch(es) exceed(s/ed)" (FEAT-D310; ident-boundary FEAT-D390). Parallel to `calls exceed` /
        // `queries exceed`. `batch exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `batches exceed` (the `es` after `batch`).
        || ((contains_phrase_after_ident_boundary(&lower, "batches exceed")
            || contains_phrase_after_ident_boundary(&lower, "batches exceeded")
            || contains_phrase_after_ident_boundary(&lower, "batch exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "token(s) exceed(s/ed)" (FEAT-D311; FEAT-D377: ident-boundary + skip when
        // the prior token is `prompt` / `input` / `total` so `micrototal prompt tokens exceed` is not
        // matched here via embedded `tokens exceed`). Parallel to `batches exceed` / `batch exceed`.
        || ((contains_tokens_exceed_subphrase_at_boundary(&lower, "tokens exceed")
            || contains_tokens_exceed_subphrase_at_boundary(&lower, "tokens exceeded")
            || contains_phrase_after_ident_boundary(&lower, "token exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "item(s) exceed(s/ed)" (FEAT-D312; ident-boundary FEAT-D390). Parallel to `tokens exceed` /
        // `token exceed`. `item exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `items exceed` (the `s` after `item`).
        || ((contains_phrase_after_ident_boundary(&lower, "items exceed")
            || contains_phrase_after_ident_boundary(&lower, "items exceeded")
            || contains_phrase_after_ident_boundary(&lower, "item exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "entr(y/ies) exceed(s/ed)" (FEAT-D313; ident-boundary FEAT-D390). Parallel to `items exceed` /
        // `item exceed`. `entry exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `entries exceed` (`entr` + `ies` vs `entry`).
        || ((contains_phrase_after_ident_boundary(&lower, "entries exceed")
            || contains_phrase_after_ident_boundary(&lower, "entries exceeded")
            || contains_phrase_after_ident_boundary(&lower, "entry exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "record(s) exceed(s/ed)" (FEAT-D314; ident-boundary FEAT-D351). Parallel
        // to `entries exceed` / `entry exceed`. `record exceed` matches present/past via `exceed`
        // prefix of `exceeds` / `exceeded` and does not substring-match plural `records exceed`
        // (the `s` after `record`). Ident-boundary so `microrecords exceed` / `metarecords exceed` /
        // `subrecord exceed` do not false-positive; `rerecord exceed` / `prerecord exceed` are
        // rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "records exceed")
            || contains_phrase_after_ident_boundary(&lower, "records exceeded")
            || contains_phrase_after_ident_boundary(&lower, "record exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "chunk(s) exceed(s/ed)" (FEAT-D315; ident-boundary FEAT-D390). Parallel to `records exceed` /
        // `record exceed`. `chunk exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `chunks exceed` (the `s` after `chunk`).
        || ((contains_phrase_after_ident_boundary(&lower, "chunks exceed")
            || contains_phrase_after_ident_boundary(&lower, "chunks exceeded")
            || contains_phrase_after_ident_boundary(&lower, "chunk exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "document(s) exceed(s/ed)" (FEAT-D316; ident-boundary FEAT-D390). Parallel to `chunks exceed` /
        // `chunk exceed`. `document exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `documents exceed` (the `s` after `document`).
        || ((contains_phrase_after_ident_boundary(&lower, "documents exceed")
            || contains_phrase_after_ident_boundary(&lower, "documents exceeded")
            || contains_phrase_after_ident_boundary(&lower, "document exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "file(s) exceed(s/ed)" (FEAT-D317; ident-boundary FEAT-D390). Parallel to `documents exceed` /
        // `document exceed`. `file exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `files exceed` (the `s` after `file`).
        || ((contains_phrase_after_ident_boundary(&lower, "files exceed")
            || contains_phrase_after_ident_boundary(&lower, "files exceeded")
            || contains_phrase_after_ident_boundary(&lower, "file exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "line(s) exceed(s/ed)" (FEAT-D318; ident-boundary FEAT-D390). Parallel to `files exceed` /
        // `file exceed`. `line exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `lines exceed` (the `s` after `line`).
        || ((contains_phrase_after_ident_boundary(&lower, "lines exceed")
            || contains_phrase_after_ident_boundary(&lower, "lines exceeded")
            || contains_phrase_after_ident_boundary(&lower, "line exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "cell(s) exceed(s/ed)" (FEAT-D319; ident-boundary FEAT-D390). Parallel to `lines exceed` /
        // `line exceed`. `cell exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `cells exceed` (the `s` after `cell`).
        || ((contains_phrase_after_ident_boundary(&lower, "cells exceed")
            || contains_phrase_after_ident_boundary(&lower, "cells exceeded")
            || contains_phrase_after_ident_boundary(&lower, "cell exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "row(s) exceed(s/ed)" (FEAT-D320). Parallel to `cells exceed` /
        // `cell exceed`. `row exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `rows exceed` (the `s` after `row`).
        // Ident-boundary on the phrases so `arrows exceed` / `throws exceed` / `arrow exceed`
        // do not match.
        || ((contains_phrase_after_ident_boundary(&lower, "rows exceed")
            || contains_phrase_after_ident_boundary(&lower, "rows exceeded")
            || contains_phrase_after_ident_boundary(&lower, "row exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "column(s) exceed(s/ed)" (FEAT-D321). Parallel to `rows exceed` /
        // `row exceed`. `column exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `columns exceed` (the `s` after `column`).
        // Ident-boundary so `microcolumns exceed` / `multicolumn exceeds` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "columns exceed")
            || contains_phrase_after_ident_boundary(&lower, "columns exceeded")
            || contains_phrase_after_ident_boundary(&lower, "column exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "table(s) exceed(s/ed)" (FEAT-D322). Parallel to `columns exceed` /
        // `column exceed`. `table exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `tables exceed` (the `s` after `table`).
        // Ident-boundary so `constables exceed` / `stable exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "tables exceed")
            || contains_phrase_after_ident_boundary(&lower, "tables exceeded")
            || contains_phrase_after_ident_boundary(&lower, "table exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "block(s) exceed(s/ed)" (FEAT-D323). Parallel to `tables exceed` /
        // `table exceed`. `block exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `blocks exceed` (the `s` after `block`).
        // Ident-boundary so `roadblocks exceed` / `roadblock exceed` / `sunblock exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "blocks exceed")
            || contains_phrase_after_ident_boundary(&lower, "blocks exceeded")
            || contains_phrase_after_ident_boundary(&lower, "block exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "segment(s) exceed(s/ed)" (FEAT-D324). Parallel to `blocks exceed` /
        // `block exceed`. `segment exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `segments exceed` (the `s` after `segment`).
        // Ident-boundary so `microsegments exceed` / `multisegment exceeds` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "segments exceed")
            || contains_phrase_after_ident_boundary(&lower, "segments exceeded")
            || contains_phrase_after_ident_boundary(&lower, "segment exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "section(s) exceed(s/ed)" (FEAT-D325). Parallel to `segments exceed` /
        // `segment exceed`. `section exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `sections exceed` (the `s` after `section`).
        // Ident-boundary so `subsections exceed` / `intersection exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "sections exceed")
            || contains_phrase_after_ident_boundary(&lower, "sections exceeded")
            || contains_phrase_after_ident_boundary(&lower, "section exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "paragraph(s) exceed(s/ed)" (FEAT-D326). Parallel to `sections exceed` /
        // `section exceed`. `paragraph exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `paragraphs exceed` (the `s` after `paragraph`).
        // Ident-boundary so `counterparagraphs exceed` / `counterparagraph exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "paragraphs exceed")
            || contains_phrase_after_ident_boundary(&lower, "paragraphs exceeded")
            || contains_phrase_after_ident_boundary(&lower, "paragraph exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "sentence(s) exceed(s/ed)" (FEAT-D327). Parallel to `paragraphs exceed` /
        // `paragraph exceed`. `sentence exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `sentences exceed` (the `s` after `sentence`).
        // Ident-boundary so `microsentences exceed` / `microsentence exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "sentences exceed")
            || contains_phrase_after_ident_boundary(&lower, "sentences exceeded")
            || contains_phrase_after_ident_boundary(&lower, "sentence exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "word(s) exceed(s/ed)" (FEAT-D328). Parallel to `sentences exceed` /
        // `sentence exceed`. `word exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `words exceed` (the `s` after `word`).
        // Ident-boundary so `buzzwords exceed` / `keywords exceed` / `buzzword exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "words exceed")
            || contains_phrase_after_ident_boundary(&lower, "words exceeded")
            || contains_phrase_after_ident_boundary(&lower, "word exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "character(s) exceed(s/ed)" (FEAT-D329). Parallel to `words exceed` /
        // `word exceed`. `character exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `characters exceed` (the `s` after `character`).
        // Ident-boundary so `megacharacters exceed` / `metacharacters exceed` / `noncharacter exceed`
        // do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "characters exceed")
            || contains_phrase_after_ident_boundary(&lower, "characters exceeded")
            || contains_phrase_after_ident_boundary(&lower, "character exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "byte(s) exceed(s/ed)" (FEAT-D330). Parallel to `characters exceed` /
        // `character exceed`. `byte exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `bytes exceed` (the `s` after `byte`).
        // Ident-boundary so `megabytes exceed` / `kilobytes exceed` / `kilobyte exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "bytes exceed")
            || contains_phrase_after_ident_boundary(&lower, "bytes exceeded")
            || contains_phrase_after_ident_boundary(&lower, "byte exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "bit(s) exceed(s/ed)" (FEAT-D331). Parallel to `bytes exceed` /
        // `byte exceed`. `bit exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `bits exceed` (the `s` after `bit`).
        // Ident-boundary so `megabits exceed` / `kilobits exceed` / `kilobit exceed` and
        // `rabbit exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "bits exceed")
            || contains_phrase_after_ident_boundary(&lower, "bits exceeded")
            || contains_phrase_after_ident_boundary(&lower, "bit exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "field(s) exceed(s/ed)" (FEAT-D332). Parallel to `bits exceed` /
        // `bit exceed`. `field exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `fields exceed` (the `s` after `field`).
        // Ident-boundary so `battlefields exceed` / `cornfields exceed` / `afield exceed` /
        // `subfield exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "fields exceed")
            || contains_phrase_after_ident_boundary(&lower, "fields exceeded")
            || contains_phrase_after_ident_boundary(&lower, "field exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "value(s) exceed(s/ed)" (FEAT-D333). Parallel to `fields exceed` /
        // `field exceed`. `value exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `values exceed` (the `s` after `value`).
        // Ident-boundary so `eigenvalues exceed` / `meanvalues exceed` / `devalue exceed` /
        // `overvalue exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "values exceed")
            || contains_phrase_after_ident_boundary(&lower, "values exceeded")
            || contains_phrase_after_ident_boundary(&lower, "value exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "key(s) exceed(s/ed)" (FEAT-D334). Parallel to `values exceed` /
        // `value exceed`. `key exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `keys exceed` (the `s` after `key`).
        // Ident-boundary so `hotkeys exceed` / `turnkeys exceed` / `monkey exceed` /
        // `passkey exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "keys exceed")
            || contains_phrase_after_ident_boundary(&lower, "keys exceeded")
            || contains_phrase_after_ident_boundary(&lower, "key exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "propert(y/ies) exceed(s/ed)" (FEAT-D335). Parallel to `keys exceed` /
        // `key exceed`. `property exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `properties exceed` (the `s` after
        // `property`). Ident-boundary so `microproperties exceed` / `subproperty exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "properties exceed")
            || contains_phrase_after_ident_boundary(&lower, "properties exceeded")
            || contains_phrase_after_ident_boundary(&lower, "property exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "schema(s) exceed(s/ed)" (FEAT-D336). Parallel to `properties exceed` /
        // `property exceed`. `schema exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `schemas exceed` (the `s` after `schema`).
        // Ident-boundary so `microschemas exceed` / `holoschemas exceed` / `subschema exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "schemas exceed")
            || contains_phrase_after_ident_boundary(&lower, "schemas exceeded")
            || contains_phrase_after_ident_boundary(&lower, "schema exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "parameter(s) exceed(s/ed)" (FEAT-D337). Parallel to `schemas exceed` /
        // `schema exceed`. `parameter exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `parameters exceed` (the `s` after
        // `parameter`). Ident-boundary so `microparameters exceed` / `metaparameters exceed` /
        // `subparameter exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "parameters exceed")
            || contains_phrase_after_ident_boundary(&lower, "parameters exceeded")
            || contains_phrase_after_ident_boundary(&lower, "parameter exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "argument(s) exceed(s/ed)" (FEAT-D338). Parallel to `parameters exceed` /
        // `parameter exceed`. `argument exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `arguments exceed` (the `s` after
        // `argument`). Ident-boundary so `microarguments exceed` / `metaarguments exceed` /
        // `subargument exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "arguments exceed")
            || contains_phrase_after_ident_boundary(&lower, "arguments exceeded")
            || contains_phrase_after_ident_boundary(&lower, "argument exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "variable(s) exceed(s/ed)" (FEAT-D339). Parallel to `arguments exceed` /
        // `argument exceed`. `variable exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `variables exceed` (the `s` after
        // `variable`). Ident-boundary so `metavariables exceed` / `hypervariables exceed` /
        // `multivariable exceed` / `subvariable exceed` do not false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "variables exceed")
            || contains_phrase_after_ident_boundary(&lower, "variables exceeded")
            || contains_phrase_after_ident_boundary(&lower, "variable exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "header(s) exceed(s/ed)" (FEAT-D340). Parallel to `variables exceed` /
        // `variable exceed`. `header exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `headers exceed` (the `s` after `header`).
        // Ident-boundary so `microheaders exceed` / `metaheaders exceed` / `subheader exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "headers exceed")
            || contains_phrase_after_ident_boundary(&lower, "headers exceeded")
            || contains_phrase_after_ident_boundary(&lower, "header exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "cookie(s) exceed(s/ed)" (FEAT-D341). Parallel to `headers exceed` /
        // `header exceed`. `cookie exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `cookies exceed` (the `s` after `cookie`).
        // Ident-boundary so `microcookies exceed` / `metacookies exceed` / `subcookie exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "cookies exceed")
            || contains_phrase_after_ident_boundary(&lower, "cookies exceeded")
            || contains_phrase_after_ident_boundary(&lower, "cookie exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "bod(y/ies) exceed(s/ed)" (FEAT-D342). Parallel to `cookies exceed` /
        // `cookie exceed`. `body exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `bodies exceed` (the `s` after `body`).
        // Ident-boundary so `microbodies exceed` / `metabodies exceed` / `subbody exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "bodies exceed")
            || contains_phrase_after_ident_boundary(&lower, "bodies exceeded")
            || contains_phrase_after_ident_boundary(&lower, "body exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "part(s) exceed(s/ed)" (FEAT-D343). Parallel to `bodies exceed` /
        // `body exceed`. `part exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `parts exceed` (the `s` after `part`).
        // Ident-boundary so `microparts exceed` / `metaparts exceed` / `subpart exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "parts exceed")
            || contains_phrase_after_ident_boundary(&lower, "parts exceeded")
            || contains_phrase_after_ident_boundary(&lower, "part exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "piece(s) exceed(s/ed)" (FEAT-D344). Parallel to `parts exceed` /
        // `part exceed`. `piece exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `pieces exceed` (the `s` after `piece`).
        // Ident-boundary so `micropieces exceed` / `metapieces exceed` / `subpiece exceed` do not
        // false-positive.
        || ((contains_phrase_after_ident_boundary(&lower, "pieces exceed")
            || contains_phrase_after_ident_boundary(&lower, "pieces exceeded")
            || contains_phrase_after_ident_boundary(&lower, "piece exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "shard(s) exceed(s/ed)" (FEAT-D345). Parallel to `pieces exceed` /
        // `piece exceed`. `shard exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `shards exceed` (the `s` after `shard`).
        // Ident-boundary so `microshards exceed` / `metashards exceed` / `subshard exceed` do not
        // false-positive; `reshard exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "shards exceed")
            || contains_phrase_after_ident_boundary(&lower, "shards exceeded")
            || contains_phrase_after_ident_boundary(&lower, "shard exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "fragment(s) exceed(s/ed)" (FEAT-D346). Parallel to `shards exceed` /
        // `shard exceed`. `fragment exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `fragments exceed` (the `s` after `fragment`).
        // Ident-boundary so `microfragments exceed` / `metafragments exceed` / `subfragment exceed` do not
        // false-positive; `refragment exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "fragments exceed")
            || contains_phrase_after_ident_boundary(&lower, "fragments exceeded")
            || contains_phrase_after_ident_boundary(&lower, "fragment exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "packet(s) exceed(s/ed)" (FEAT-D347). Parallel to `fragments exceed` /
        // `fragment exceed`. `packet exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `packets exceed` (the `s` after `packet`).
        // Ident-boundary so `micropackets exceed` / `metapackets exceed` / `subpacket exceed` do not
        // false-positive; `repacket exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "packets exceed")
            || contains_phrase_after_ident_boundary(&lower, "packets exceeded")
            || contains_phrase_after_ident_boundary(&lower, "packet exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "frame(s) exceed(s/ed)" (FEAT-D348). Parallel to `packets exceed` /
        // `packet exceed`. `frame exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `frames exceed` (the `s` after `frame`).
        // Ident-boundary so `microframes exceed` / `metaframes exceed` / `subframe exceed` do not
        // false-positive; `reframe exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "frames exceed")
            || contains_phrase_after_ident_boundary(&lower, "frames exceeded")
            || contains_phrase_after_ident_boundary(&lower, "frame exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "sample(s) exceed(s/ed)" (FEAT-D349). Parallel to `frames exceed` /
        // `frame exceed`. `sample exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `samples exceed` (the `s` after `sample`).
        // Ident-boundary so `microsamples exceed` / `metasamples exceed` / `subsample exceed` do not
        // false-positive; `resample exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "samples exceed")
            || contains_phrase_after_ident_boundary(&lower, "samples exceeded")
            || contains_phrase_after_ident_boundary(&lower, "sample exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "observation(s) exceed(s/ed)" (FEAT-D350). Parallel to `samples exceed` /
        // `sample exceed`. `observation exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `observations exceed` (the `s` after `observation`).
        // Ident-boundary so `microobservations exceed` / `metaobservations exceed` / `subobservation exceed` do not
        // false-positive; `preobservation exceed` is rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "observations exceed")
            || contains_phrase_after_ident_boundary(&lower, "observations exceeded")
            || contains_phrase_after_ident_boundary(&lower, "observation exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "event(s) exceed(s/ed)" (FEAT-D352). Parallel to `observations exceed` /
        // `observation exceed`. `event exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `events exceed` (the `s` after `event`).
        // Ident-boundary so `microevents exceed` / `metaevents exceed` / `subevent exceed` do not
        // false-positive; `preevent exceed` is rejected the same way (and embedded `event exceed` in
        // `prevent exceed` is rejected by the left boundary).
        || ((contains_phrase_after_ident_boundary(&lower, "events exceed")
            || contains_phrase_after_ident_boundary(&lower, "events exceeded")
            || contains_phrase_after_ident_boundary(&lower, "event exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "trace(s) exceed(s/ed)" (FEAT-D354). Parallel to `events exceed` /
        // `event exceed`. `trace exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `traces exceed` (the `s` after `trace`).
        // Ident-boundary so `microtraces exceed` / `metatraces exceed` / `subtrace exceed` do not
        // false-positive; `pretrace exceed` and `retrace exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "traces exceed")
            || contains_phrase_after_ident_boundary(&lower, "traces exceeded")
            || contains_phrase_after_ident_boundary(&lower, "trace exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "span(s) exceed(s/ed)" (FEAT-D355). Parallel to `traces exceed` /
        // `trace exceed`. `span exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `spans exceed` (the `s` after `span`).
        // Ident-boundary so `microspans exceed` / `metaspans exceed` / `subspan exceed` do not
        // false-positive; `prespan exceed` and `respan exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "spans exceed")
            || contains_phrase_after_ident_boundary(&lower, "spans exceeded")
            || contains_phrase_after_ident_boundary(&lower, "span exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "attribute(s) exceed(s/ed)" (FEAT-D356). Parallel to `spans exceed` /
        // `span exceed`. `attribute exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `attributes exceed` (the `s` after
        // `attribute`). Ident-boundary so `microattributes exceed` / `metaattributes exceed` /
        // `subattribute exceed` do not false-positive; `preattribute exceed` and `reattribute exceed`
        // are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "attributes exceed")
            || contains_phrase_after_ident_boundary(&lower, "attributes exceeded")
            || contains_phrase_after_ident_boundary(&lower, "attribute exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "link(s) exceed(s/ed)" (FEAT-D357). Parallel to `attributes exceed` /
        // `attribute exceed`. `link exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `links exceed` (the `s` after `link`).
        // Ident-boundary so `microlinks exceed` / `metalinks exceed` / `sublink exceed` do not
        // false-positive; `prelink exceed` and `relink exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "links exceed")
            || contains_phrase_after_ident_boundary(&lower, "links exceeded")
            || contains_phrase_after_ident_boundary(&lower, "link exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "scope(s) exceed(s/ed)" (FEAT-D359). Parallel to `links exceed` /
        // `link exceed`. `scope exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `scopes exceed` (the `s` after `scope`).
        // Ident-boundary so `microscopes exceed` / `metascopes exceed` / `subscope exceed` do not
        // false-positive; `prescope exceed` and `rescope exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "scopes exceed")
            || contains_phrase_after_ident_boundary(&lower, "scopes exceeded")
            || contains_phrase_after_ident_boundary(&lower, "scope exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "resource(s) exceed(s/ed)" (FEAT-D360). Parallel to `scopes exceed` /
        // `scope exceed`. `resource exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `resources exceed` (the `s` after `resource`).
        // Ident-boundary so `microresources exceed` / `metaresources exceed` / `subresource exceed`
        // do not false-positive; `preresource exceed` and `reresource exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "resources exceed")
            || contains_phrase_after_ident_boundary(&lower, "resources exceeded")
            || contains_phrase_after_ident_boundary(&lower, "resource exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "metric(s) exceed(s/ed)" (FEAT-D361). Parallel to `resources exceed` /
        // `resource exceed`. `metric exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `metrics exceed` (the `s` after `metric`).
        // Ident-boundary so `micrometrics exceed` / `metametrics exceed` / `submetric exceed`
        // do not false-positive; `premetric exceed` and `remetric exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "metrics exceed")
            || contains_phrase_after_ident_boundary(&lower, "metrics exceeded")
            || contains_phrase_after_ident_boundary(&lower, "metric exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "dimension(s) exceed(s/ed)" (FEAT-D362). Parallel to `metrics exceed` /
        // `metric exceed`. `dimension exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `dimensions exceed` (the `s` after `dimension`).
        // Ident-boundary so `microdimensions exceed` / `metadimensions exceed` / `subdimension exceed`
        // do not false-positive; `predimension exceed` and `redimension exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "dimensions exceed")
            || contains_phrase_after_ident_boundary(&lower, "dimensions exceeded")
            || contains_phrase_after_ident_boundary(&lower, "dimension exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "tensor(s) exceed(s/ed)" (FEAT-D363). Parallel to `dimensions exceed` /
        // `dimension exceed`. `tensor exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `tensors exceed` (the `s` after `tensor`).
        // Ident-boundary so `microtensors exceed` / `metatensors exceed` / `subtensor exceed`
        // do not false-positive; `pretensor exceed` and `retensor exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "tensors exceed")
            || contains_phrase_after_ident_boundary(&lower, "tensors exceeded")
            || contains_phrase_after_ident_boundary(&lower, "tensor exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "activation(s) exceed(s/ed)" (FEAT-D364). Parallel to `tensors exceed` /
        // `tensor exceed`. `activation exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `activations exceed` (the `s` after `activation`).
        // Ident-boundary so `microactivations exceed` / `metaactivations exceed` / `subactivation exceed`
        // do not false-positive; `preactivation exceed` and `reactivation exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "activations exceed")
            || contains_phrase_after_ident_boundary(&lower, "activations exceeded")
            || contains_phrase_after_ident_boundary(&lower, "activation exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "gradient(s) exceed(s/ed)" (FEAT-D365). Parallel to `activations exceed` /
        // `activation exceed`. `gradient exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `gradients exceed` (the `s` after `gradient`).
        // Ident-boundary so `microgradients exceed` / `metagradients exceed` / `subgradient exceed`
        // do not false-positive; `pregradient exceed` and `regradient exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "gradients exceed")
            || contains_phrase_after_ident_boundary(&lower, "gradients exceeded")
            || contains_phrase_after_ident_boundary(&lower, "gradient exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "weight(s) exceed(s/ed)" (FEAT-D366). Parallel to `gradients exceed` /
        // `gradient exceed`. `weight exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `weights exceed` (the `s` after `weight`).
        // Ident-boundary so `microweights exceed` / `metaweights exceed` / `subweight exceed`
        // do not false-positive; `preweight exceed` and `reweight exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "weights exceed")
            || contains_phrase_after_ident_boundary(&lower, "weights exceeded")
            || contains_phrase_after_ident_boundary(&lower, "weight exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "bias(es) exceed(s/ed)" (FEAT-D367). Parallel to `weights exceed` /
        // `weight exceed`. `bias exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `biases exceed` (the `s` after `bias`).
        // Ident-boundary so `microbiases exceed` / `metabiases exceed` / `subbias exceed`
        // do not false-positive; `prebias exceed` and `rebias exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "biases exceed")
            || contains_phrase_after_ident_boundary(&lower, "biases exceeded")
            || contains_phrase_after_ident_boundary(&lower, "bias exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "layer(s) exceed(s/ed)" (FEAT-D368). Parallel to `biases exceed` /
        // `bias exceed`. `layer exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `layers exceed` (the `s` after `layer`).
        // Ident-boundary so `microlayers exceed` / `metalayers exceed` / `sublayer exceed`
        // do not false-positive; `prelayer exceed` and `relayer exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "layers exceed")
            || contains_phrase_after_ident_boundary(&lower, "layers exceeded")
            || contains_phrase_after_ident_boundary(&lower, "layer exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "head(s) exceed(s/ed)" (FEAT-D369). Parallel to `layers exceed` /
        // `layer exceed`. `head exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `heads exceed` (the `s` after `head`).
        // Ident-boundary so `microheads exceed` / `metaheads exceed` / `subhead exceed`
        // do not false-positive; `prehead exceed` and `rehead exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "heads exceed")
            || contains_phrase_after_ident_boundary(&lower, "heads exceeded")
            || contains_phrase_after_ident_boundary(&lower, "head exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "position(s) exceed(s/ed)" (FEAT-D370). Parallel to `heads exceed` /
        // `head exceed`. `position exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `positions exceed` (the `s` after
        // `position`). Ident-boundary so `micropositions exceed` / `metapositions exceed` /
        // `subposition exceed` do not false-positive; `preposition exceed` and `reposition exceed`
        // are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "positions exceed")
            || contains_phrase_after_ident_boundary(&lower, "positions exceeded")
            || contains_phrase_after_ident_boundary(&lower, "position exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "embedding(s) exceed(s/ed)" (FEAT-D371). Parallel to `positions exceed` /
        // `position exceed`. `embedding exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `embeddings exceed` (the `s` after
        // `embedding`). Ident-boundary so `microembeddings exceed` / `metaembeddings exceed` /
        // `subembedding exceed` do not false-positive; `preembedding exceed` and `reembedding exceed`
        // are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "embeddings exceed")
            || contains_phrase_after_ident_boundary(&lower, "embeddings exceeded")
            || contains_phrase_after_ident_boundary(&lower, "embedding exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "logit(s) exceed(s/ed)" (FEAT-D372). Parallel to `embeddings exceed` /
        // `embedding exceed`. `logit exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `logits exceed` (the `s` after
        // `logit`). Ident-boundary so `micrologits exceed` / `metalogits exceed` /
        // `sublogit exceed` do not false-positive; `prelogit exceed` and `relogit exceed`
        // are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "logits exceed")
            || contains_phrase_after_ident_boundary(&lower, "logits exceeded")
            || contains_phrase_after_ident_boundary(&lower, "logit exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "probabilit(y|ies) exceed(s/ed)" (FEAT-D373). Parallel to `logits exceed` /
        // `logit exceed`. `probability exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `probabilities exceed` (no `probability` + space
        // + `exceed` inside that spelling). Ident-boundary so
        // `microprobabilities exceed` / `metaprobabilities exceed` / `subprobability exceed` do not
        // false-positive; `preprobability exceed` and `reprobability exceed` are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "probabilities exceed")
            || contains_phrase_after_ident_boundary(&lower, "probabilities exceeded")
            || contains_phrase_after_ident_boundary(&lower, "probability exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "logprob(s) exceed(s/ed)" (FEAT-D374). Parallel to `probabilities exceed` /
        // `probability exceed`. `logprob exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `logprobs exceed` (the `s` after
        // `logprob`). Ident-boundary so `micrologprobs exceed` / `metalogprobs exceed` /
        // `sublogprob exceed` do not false-positive; `prelogprob exceed` and `relogprob exceed`
        // are rejected the same way.
        || ((contains_phrase_after_ident_boundary(&lower, "logprobs exceed")
            || contains_phrase_after_ident_boundary(&lower, "logprobs exceeded")
            || contains_phrase_after_ident_boundary(&lower, "logprob exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "string(s) exceed(s/ed)" (FEAT-D392). Parallel to `bytes exceed` /
        // `byte exceed`. `string exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `strings exceed` (the `s` after `string`).
        // Ident-boundary so `microstrings exceed` / `metastrings exceed` / `substring exceed`
        // do not false-positive on `strings exceed` / `string exceed` (embedded `string exceed`
        // in `substring exceed` is rejected: ident continuation before `string`). Same explicit
        // context-slot phrases as `messages exceed`. Negatives: HTTP `strings exceed` rate limits,
        // max-string / UTF-8 caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "strings exceed")
            || contains_phrase_after_ident_boundary(&lower, "strings exceeded")
            || contains_phrase_after_ident_boundary(&lower, "string exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "array(s) exceed(s/ed)" (FEAT-D393). Parallel to `strings exceed` /
        // `string exceed`. `array exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `arrays exceed` (the `s` after `array`).
        // Ident-boundary so `microarrays exceed` / `metaarrays exceed` / `subarray exceed` do not
        // false-positive; `prearray exceed` and `rearray exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `arrays exceed` rate
        // limits, max-depth / dimension caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "arrays exceed")
            || contains_phrase_after_ident_boundary(&lower, "arrays exceeded")
            || contains_phrase_after_ident_boundary(&lower, "array exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "object(s) exceed(s/ed)" (FEAT-D394). Parallel to `arrays exceed` /
        // `array exceed`. `object exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `objects exceed` (the `s` after `object`).
        // Ident-boundary so `microobjects exceed` / `metaobjects exceed` / `subobject exceed` do not
        // false-positive; `preobject exceed` and `reobject exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `objects exceed` rate
        // limits, per-object / reference caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "objects exceed")
            || contains_phrase_after_ident_boundary(&lower, "objects exceeded")
            || contains_phrase_after_ident_boundary(&lower, "object exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "element(s) exceed(s/ed)" (FEAT-D395). Parallel to `objects exceed` /
        // `object exceed`. `element exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `elements exceed` (the `s` after `element`).
        // Ident-boundary so `microelements exceed` / `metaelements exceed` / `subelement exceed` do not
        // false-positive; `preelement exceed` and `reelement exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `elements exceed` rate
        // limits, max DOM / tensor element counts, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "elements exceed")
            || contains_phrase_after_ident_boundary(&lower, "elements exceeded")
            || contains_phrase_after_ident_boundary(&lower, "element exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "node(s) exceed(s/ed)" (FEAT-D396). Parallel to `elements exceed` /
        // `element exceed`. `node exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `nodes exceed` (the `s` after `node`).
        // Ident-boundary so `micronodes exceed` / `metanodes exceed` / `subnode exceed` do not
        // false-positive; `prenode exceed` and `renode exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `nodes exceed` rate
        // limits, max graph / DOM node counts, cluster membership caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "nodes exceed")
            || contains_phrase_after_ident_boundary(&lower, "nodes exceeded")
            || contains_phrase_after_ident_boundary(&lower, "node exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "edge(s) exceed(s/ed)" (FEAT-D397). Parallel to `nodes exceed` /
        // `node exceed`. `edge exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `edges exceed` (the `s` after `edge`).
        // Ident-boundary so `microedges exceed` / `metaedges exceed` / `subedge exceed` do not
        // false-positive; `preedge exceed` and `reedge exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `edges exceed` rate
        // limits, max graph adjacency / fan-out caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "edges exceed")
            || contains_phrase_after_ident_boundary(&lower, "edges exceeded")
            || contains_phrase_after_ident_boundary(&lower, "edge exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "vert(ices|ex) exceed(s/ed)" (FEAT-D398). Parallel to `edges exceed` /
        // `edge exceed`. `vertex exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `vertices exceed` (after `vert` the plural
        // has `ices`, not `ex` + space + `exceed`). Ident-boundary so `microvertices exceed` /
        // `metavertices exceed` / `subvertex exceed` do not false-positive; `prevertex exceed` and
        // `revertex exceed` are rejected the same way. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `vertices exceed` rate limits, mesh / graph vertex
        // caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "vertices exceed")
            || contains_phrase_after_ident_boundary(&lower, "vertices exceeded")
            || contains_phrase_after_ident_boundary(&lower, "vertex exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "face(s) exceed(s/ed)" (FEAT-D399). Parallel to `vertices exceed` /
        // `vertex exceed`. `face exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `faces exceed` (the `s` after `face`).
        // Ident-boundary so `microfaces exceed` / `metafaces exceed` / `subface exceed` do not
        // false-positive; `preface exceed` and `reface exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `faces exceed` rate
        // limits, mesh / polygon caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "faces exceed")
            || contains_phrase_after_ident_boundary(&lower, "faces exceeded")
            || contains_phrase_after_ident_boundary(&lower, "face exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "triangle(s) exceed(s/ed)" (FEAT-D400). Parallel to `faces exceed` /
        // `face exceed`. `triangle exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `triangles exceed` (the `s` after `triangle`).
        // Ident-boundary so `microtriangles exceed` / `metatriangles exceed` / `subtriangle exceed`
        // do not false-positive; `pretriangle exceed` and `retriangle exceed` are rejected the same
        // way. Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `triangles
        // exceed` rate limits, mesh / index caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "triangles exceed")
            || contains_phrase_after_ident_boundary(&lower, "triangles exceeded")
            || contains_phrase_after_ident_boundary(&lower, "triangle exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "polygon(s) exceed(s/ed)" (FEAT-D401). Parallel to `triangles exceed` /
        // `triangle exceed`. `polygon exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `polygons exceed` (the `s` after `polygon`).
        // Ident-boundary so `micropolygons exceed` / `metapolygons exceed` / `subpolygon exceed`
        // do not false-positive; `prepolygon exceed` and `repolygon exceed` are rejected the same
        // way. Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `polygons
        // exceed` rate limits, GIS / mesh ring caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "polygons exceed")
            || contains_phrase_after_ident_boundary(&lower, "polygons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "polygon exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "mesh(es) exceed(s/ed)" (FEAT-D402). Parallel to `polygons exceed` /
        // `polygon exceed`. `mesh exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `meshes exceed` (the `s` after `mesh`).
        // Ident-boundary so `micromeshes exceed` / `metameshes exceed` / `submesh exceed` do not
        // false-positive; `premesh exceed` and `remesh exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `meshes exceed` rate
        // limits, per-scene / draw-call caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "meshes exceed")
            || contains_phrase_after_ident_boundary(&lower, "meshes exceeded")
            || contains_phrase_after_ident_boundary(&lower, "mesh exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "voxel(s) exceed(s/ed)" (FEAT-D403). Parallel to `meshes exceed` /
        // `mesh exceed`. `voxel exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `voxels exceed` (the `s` after `voxel`).
        // Ident-boundary so `microvoxels exceed` / `metavoxels exceed` / `subvoxel exceed` do not
        // false-positive; `prevoxel exceed` and `revoxel exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `voxels exceed` rate
        // limits, per-chunk / grid-resolution caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "voxels exceed")
            || contains_phrase_after_ident_boundary(&lower, "voxels exceeded")
            || contains_phrase_after_ident_boundary(&lower, "voxel exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "particle(s) exceed(s/ed)" (FEAT-D404). Parallel to `voxels exceed` /
        // `voxel exceed`. `particle exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `particles exceed` (the `s` after `particle`).
        // Ident-boundary so `microparticles exceed` / `metaparticles exceed` / `subparticle exceed` do not
        // false-positive; `preparticle exceed` and `reparticle exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `particles exceed` rate
        // limits, per-emitter / simulation caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "particles exceed")
            || contains_phrase_after_ident_boundary(&lower, "particles exceeded")
            || contains_phrase_after_ident_boundary(&lower, "particle exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "molecule(s) exceed(s/ed)" (FEAT-D405). Parallel to `particles exceed` /
        // `particle exceed`. `molecule exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `molecules exceed` (the `s` after `molecule`).
        // Ident-boundary so `micromolecules exceed` / `metamolecules exceed` / `submolecule exceed` do not
        // false-positive; `premolecule exceed` and `remolecule exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `molecules exceed` rate
        // limits, per-reaction / structure caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "molecules exceed")
            || contains_phrase_after_ident_boundary(&lower, "molecules exceeded")
            || contains_phrase_after_ident_boundary(&lower, "molecule exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "atom(s) exceed(s/ed)" (FEAT-D406). Parallel to `molecules exceed` /
        // `molecule exceed`. `atom exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `atoms exceed` (the `s` after `atom`).
        // Ident-boundary so `microatoms exceed` / `metaatoms exceed` / `subatom exceed` do not
        // false-positive; `preatom exceed` and `reatom exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `atoms exceed` rate
        // limits, per-structure / basis-set caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "atoms exceed")
            || contains_phrase_after_ident_boundary(&lower, "atoms exceeded")
            || contains_phrase_after_ident_boundary(&lower, "atom exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "ion(s) exceed(s/ed)" (FEAT-D407). Parallel to `atoms exceed` /
        // `atom exceed`. `ion exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `ions exceed` (the `s` after `ion`).
        // Ident-boundary so `microions exceed` / `metaions exceed` / `subion exceed` do not
        // false-positive; `preion exceed` and `reion exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `ions exceed` rate
        // limits, per-species / plasma caps, etc. without slot wording. `million exceed` does not
        // match `ion exceed` (the `l` before `ion` breaks the left boundary).
        || ((contains_phrase_after_ident_boundary(&lower, "ions exceed")
            || contains_phrase_after_ident_boundary(&lower, "ions exceeded")
            || contains_phrase_after_ident_boundary(&lower, "ion exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "electron(s) exceed(s/ed)" (FEAT-D408). Parallel to `ions exceed` /
        // `ion exceed`. `electron exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `electrons exceed` (the `s` after `electron`).
        // Ident-boundary so `microelectrons exceed` / `metaelectrons exceed` / `subelectron exceed` do not
        // false-positive; `preelectron exceed` and `reelectron exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `electrons exceed` rate
        // limits, per-orbital / beam caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "electrons exceed")
            || contains_phrase_after_ident_boundary(&lower, "electrons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "electron exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "proton(s) exceed(s/ed)" (FEAT-D409). Parallel to `electrons exceed` /
        // `electron exceed`. `proton exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `protons exceed` (the `s` after `proton`).
        // Ident-boundary so `microprotons exceed` / `metaprotons exceed` / `subproton exceed` do not
        // false-positive; `preproton exceed` and `reproton exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `protons exceed` rate
        // limits, per-nucleus / charge caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "protons exceed")
            || contains_phrase_after_ident_boundary(&lower, "protons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "proton exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "neutron(s) exceed(s/ed)" (FEAT-D410). Parallel to `protons exceed` /
        // `proton exceed`. `neutron exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `neutrons exceed` (the `s` after `neutron`).
        // Ident-boundary so `microneutrons exceed` / `metaneutrons exceed` / `subneutron exceed` do not
        // false-positive; `preneutron exceed` and `reneutron exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `neutrons exceed` rate
        // limits, per-nucleus / cross-section caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "neutrons exceed")
            || contains_phrase_after_ident_boundary(&lower, "neutrons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "neutron exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "quark(s) exceed(s/ed)" (FEAT-D411). Parallel to `neutrons exceed` /
        // `neutron exceed`. `quark exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `quarks exceed` (the `s` after `quark`).
        // Ident-boundary so `microquarks exceed` / `metaquarks exceed` / `subquark exceed` do not
        // false-positive; `prequark exceed` and `requark exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `quarks exceed` rate
        // limits, per-flavor / QCD caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "quarks exceed")
            || contains_phrase_after_ident_boundary(&lower, "quarks exceeded")
            || contains_phrase_after_ident_boundary(&lower, "quark exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "gluon(s) exceed(s/ed)" (FEAT-D412). Parallel to `quarks exceed` /
        // `quark exceed`. `gluon exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `gluons exceed` (the `s` after `gluon`).
        // Ident-boundary so `microgluons exceed` / `metagluons exceed` / `subgluon exceed` do not
        // false-positive; `pregluon exceed` and `regluon exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `gluons exceed` rate
        // limits, per-color / gauge-link caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "gluons exceed")
            || contains_phrase_after_ident_boundary(&lower, "gluons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "gluon exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "boson(s) exceed(s/ed)" (FEAT-D413). Parallel to `gluons exceed` /
        // `gluon exceed`. `boson exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `bosons exceed` (the `s` after `boson`).
        // Ident-boundary so `microbosons exceed` / `metabosons exceed` / `subboson exceed` do not
        // false-positive; `preboson exceed` and `reboson exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `bosons exceed` rate
        // limits, per-mode / occupancy caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "bosons exceed")
            || contains_phrase_after_ident_boundary(&lower, "bosons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "boson exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "lepton(s) exceed(s/ed)" (FEAT-D414). Parallel to `bosons exceed` /
        // `boson exceed`. `lepton exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `leptons exceed` (the `s` after `lepton`).
        // Ident-boundary so `microleptons exceed` / `metaleptons exceed` / `sublepton exceed` do not
        // false-positive; `prelepton exceed` and `relepton exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `leptons exceed` rate
        // limits, per-generation / family caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "leptons exceed")
            || contains_phrase_after_ident_boundary(&lower, "leptons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "lepton exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "hadron(s) exceed(s/ed)" (FEAT-D415). Parallel to `leptons exceed` /
        // `lepton exceed`. `hadron exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `hadrons exceed` (the `s` after `hadron`).
        // Ident-boundary so `microhadrons exceed` / `metahadrons exceed` / `subhadron exceed` do not
        // false-positive; `prehadron exceed` and `rehadron exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `hadrons exceed` rate
        // limits, per-jet / multiplicity caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "hadrons exceed")
            || contains_phrase_after_ident_boundary(&lower, "hadrons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "hadron exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "photon(s) exceed(s/ed)" (FEAT-D416). Parallel to `hadrons exceed` /
        // `hadron exceed`. `photon exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `photons exceed` (the `s` after `photon`).
        // Ident-boundary so `microphotons exceed` / `metaphotons exceed` / `subphoton exceed` do not
        // false-positive; `prephoton exceed` and `rephoton exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `photons exceed` rate
        // limits, per-beam / fluence caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "photons exceed")
            || contains_phrase_after_ident_boundary(&lower, "photons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "photon exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "phonon(s) exceed(s/ed)" (FEAT-D417). Parallel to `photons exceed` /
        // `photon exceed`. `phonon exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `phonons exceed` (the `s` after `phonon`).
        // Ident-boundary so `microphonons exceed` / `metaphonons exceed` / `subphonon exceed` do not
        // false-positive; `prephonon exceed` and `rephonon exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `phonons exceed` rate
        // limits, per-branch / thermal-occupancy caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "phonons exceed")
            || contains_phrase_after_ident_boundary(&lower, "phonons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "phonon exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "exciton(s) exceed(s/ed)" (FEAT-D418). Parallel to `phonons exceed` /
        // `phonon exceed`. `exciton exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `excitons exceed` (the `s` after `exciton`).
        // Ident-boundary so `microexcitons exceed` / `metaexcitons exceed` / `subexciton exceed` do not
        // false-positive; `preexciton exceed` and `reexciton exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `excitons exceed` rate
        // limits, per-well / occupancy caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "excitons exceed")
            || contains_phrase_after_ident_boundary(&lower, "excitons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "exciton exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "polaron(s) exceed(s/ed)" (FEAT-D419). Parallel to `excitons exceed` /
        // `exciton exceed`. `polaron exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `polarons exceed` (the `s` after `polaron`).
        // Ident-boundary so `micropolarons exceed` / `metapolarons exceed` / `subpolaron exceed` do not
        // false-positive; `prepolaron exceed` and `repolaron exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `polarons exceed` rate
        // limits, per-lattice / deformation caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "polarons exceed")
            || contains_phrase_after_ident_boundary(&lower, "polarons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "polaron exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "plasmon(s) exceed(s/ed)" (FEAT-D420). Parallel to `polarons exceed` /
        // `polaron exceed`. `plasmon exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `plasmons exceed` (the `s` after `plasmon`).
        // Ident-boundary so `microplasmons exceed` / `metaplasmons exceed` / `subplasmon exceed` do not
        // false-positive; `preplasmon exceed` and `replasmon exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `plasmons exceed` rate
        // limits, per-mode / near-field caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "plasmons exceed")
            || contains_phrase_after_ident_boundary(&lower, "plasmons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "plasmon exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "soliton(s) exceed(s/ed)" (FEAT-D421). Parallel to `plasmons exceed` /
        // `plasmon exceed`. `soliton exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `solitons exceed` (the `s` after `soliton`).
        // Ident-boundary so `microsolitons exceed` / `metasolitons exceed` / `subsoliton exceed` do not
        // false-positive; `presoliton exceed` and `resoliton exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `solitons exceed` rate
        // limits, per-pulse / nonlinear-mode caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "solitons exceed")
            || contains_phrase_after_ident_boundary(&lower, "solitons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "soliton exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "instanton(s) exceed(s/ed)" (FEAT-D422). Parallel to `solitons exceed` /
        // `soliton exceed`. `instanton exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `instantons exceed` (the `s` after `instanton`).
        // Ident-boundary so `microinstantons exceed` / `metainstantons exceed` / `subinstanton exceed` do not
        // false-positive; `preinstanton exceed` and `reinstanton exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `instantons exceed` rate
        // limits, per-sector / gauge-orbit caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "instantons exceed")
            || contains_phrase_after_ident_boundary(&lower, "instantons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "instanton exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "skyrmion(s) exceed(s/ed)" (FEAT-D423). Parallel to `instantons exceed` /
        // `instanton exceed`. `skyrmion exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `skyrmions exceed` (the `s` after `skyrmion`).
        // Ident-boundary so `microskyrmions exceed` / `metaskyrmions exceed` / `subskyrmion exceed` do not
        // false-positive; `preskyrmion exceed` and `reskyrmion exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `skyrmions exceed` rate
        // limits, per-track / topological-charge caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "skyrmions exceed")
            || contains_phrase_after_ident_boundary(&lower, "skyrmions exceeded")
            || contains_phrase_after_ident_boundary(&lower, "skyrmion exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "magnon(s) exceed(s/ed)" (FEAT-D424). Parallel to `skyrmions exceed` /
        // `skyrmion exceed`. `magnon exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `magnons exceed` (the `s` after `magnon`).
        // Ident-boundary so `micromagnons exceed` / `metamagnons exceed` / `submagnon exceed` do not
        // false-positive; `premagnon exceed` and `remagnon exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `magnons exceed` rate
        // limits, per-branch / spin-wave caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "magnons exceed")
            || contains_phrase_after_ident_boundary(&lower, "magnons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "magnon exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "roton(s) exceed(s/ed)" (FEAT-D425). Parallel to `magnons exceed` /
        // `magnon exceed`. `roton exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `rotons exceed` (the `s` after `roton`).
        // Ident-boundary so `microrotons exceed` / `metarotons exceed` / `subroton exceed` do not
        // false-positive; `preroton exceed` and `reroton exceed` are rejected the same way. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `rotons exceed` rate
        // limits, per-branch / Landau-level caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "rotons exceed")
            || contains_phrase_after_ident_boundary(&lower, "rotons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "roton exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "anyon(s) exceed(s/ed)" (FEAT-D426). Parallel to `rotons exceed` /
        // `roton exceed`. `anyon exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `anyons exceed` (the `s` after `anyon`).
        // Ident-boundary so `microanyons exceed` / `metaanyons exceed` / `subanyon exceed` do not
        // false-positive; `preanyon exceed` and `reanyon exceed` are rejected the same way; `canyon
        // exceed` does not match `anyon exceed` (the `c` before `anyon` breaks the boundary). Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `anyons exceed` rate
        // limits, per-braid / fusion-rule caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "anyons exceed")
            || contains_phrase_after_ident_boundary(&lower, "anyons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "anyon exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "fluxon(s) exceed(s/ed)" (FEAT-D427). Parallel to `anyons exceed` /
        // `anyon exceed`. `fluxon exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `fluxons exceed` (the `s` after `fluxon`).
        // Ident-boundary so `microfluxons exceed` / `metafluxons exceed` / `subfluxon exceed` do not
        // false-positive; `prefluxon exceed` and `refluxon exceed` are rejected the same way.
        // Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `fluxons exceed`
        // rate limits, per-Josephson / flux-quantum caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "fluxons exceed")
            || contains_phrase_after_ident_boundary(&lower, "fluxons exceeded")
            || contains_phrase_after_ident_boundary(&lower, "fluxon exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "vortices / vortex exceed(s/ed)" (FEAT-D428). Parallel to `fluxons exceed` /
        // `fluxon exceed`. `vortex exceed` matches present/past via `exceed` prefix of `exceeds` /
        // `exceeded` and does not substring-match plural `vortices exceed` (the plural is `vortices`,
        // not `vortex`). Ident-boundary so `microvortices exceed`
        // / `metavortices exceed` / `subvortex exceed` do not false-positive; `prevortex exceed` and
        // `revortex exceed` are rejected the same way. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `vortices exceed` rate limits, per-cell / circulation
        // caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "vortices exceed")
            || contains_phrase_after_ident_boundary(&lower, "vortices exceeded")
            || contains_phrase_after_ident_boundary(&lower, "vortex exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // "message/input(s) … too long" (distinct from `prompt too long` already handled above).
        // Same context-slot guard as `messages exceed` (FEAT-D295) so incidental `model context`
        // copy does not match non-slot errors. Plural `inputs are/were` (FEAT-D302) parallels
        // `messages are/were` and does not substring-match singular `input is/was`.
        // Ident-boundary (FEAT-D391): `submessage is too long` / `micromessage is too long` /
        // `subinput is too long` do not embed the phrases at a word boundary.
        || ((contains_phrase_after_ident_boundary(&lower, "message is too long")
            || contains_phrase_after_ident_boundary(&lower, "messages are too long")
            || contains_phrase_after_ident_boundary(&lower, "message was too long")
            || contains_phrase_after_ident_boundary(&lower, "messages were too long")
            || contains_phrase_after_ident_boundary(&lower, "input is too long")
            || contains_phrase_after_ident_boundary(&lower, "input was too long")
            || contains_phrase_after_ident_boundary(&lower, "inputs are too long")
            || contains_phrase_after_ident_boundary(&lower, "inputs were too long"))
            && explicit_context_slot_after_ident_boundary(&lower))
}

/// Check whether an Ollama error indicates message role/ordering conflict.
fn is_role_ordering_error(lower: &str) -> bool {
    (lower.contains("role") && (lower.contains("alternate") || lower.contains("ordering")))
        || lower.contains("incorrect role")
        || lower.contains("roles must alternate")
        || lower.contains("expected role")
        || (lower.contains("invalid") && lower.contains("role"))
}

/// Check whether an Ollama error indicates corrupted session / missing tool input.
fn is_corrupted_session_error(lower: &str) -> bool {
    (lower.contains("tool") && lower.contains("missing"))
        || lower.contains("invalid message")
        || lower.contains("malformed")
        || (lower.contains("tool_calls") && lower.contains("expected"))
}

/// Rewrite a raw Ollama/pipeline error into a short, user-friendly message.
///
/// Maps known error categories to actionable text that suggests starting a
/// new topic (matching the wording users already know from session reset).
/// Returns `None` when the error does not match any known pattern, so callers
/// can fall back to their existing formatting.
pub(crate) fn sanitize_ollama_error_for_user(raw: &str) -> Option<String> {
    let lower = raw.to_lowercase();

    let friendly = if is_context_overflow_error(raw) {
        Some(
            "The conversation got too long for the model's context window. \
             Try starting a new topic or using a model with a larger context."
                .to_string(),
        )
    } else if is_role_ordering_error(&lower) {
        Some(
            "Message ordering conflict — please try again. \
             If this keeps happening, start a new topic to reset the conversation."
                .to_string(),
        )
    } else if is_corrupted_session_error(&lower) {
        Some(
            "The conversation history looks corrupted. \
             Start a new topic to begin a fresh session."
                .to_string(),
        )
    } else {
        None
    };

    if friendly.is_some() {
        tracing::debug!("Sanitized Ollama error for user — raw: {}", raw);
    }

    friendly
}

/// Truncate oversized tool-result messages in the conversation to `max_chars_per_result`.
///
/// Only truncates assistant/user/system messages whose content exceeds `max_chars_per_result`
/// and that look like tool results (heuristic: not the very first system prompt, and not
/// messages that are the user's original question).
///
/// Returns the number of messages that were truncated.
pub(crate) fn truncate_oversized_tool_results(
    messages: &mut [crate::ollama::ChatMessage],
    max_chars_per_result: usize,
) -> usize {
    let mut truncated_count = 0usize;
    for (i, msg) in messages.iter_mut().enumerate() {
        // Skip the first message (system prompt) — it contains the agent instructions.
        if i == 0 && msg.role == "system" {
            continue;
        }
        let char_count = msg.content.chars().count();
        if char_count <= max_chars_per_result {
            continue;
        }
        let truncated_body = truncate_at_boundary(&msg.content, max_chars_per_result);
        msg.content = format!(
            "{}\n\n[truncated from {} to {} chars due to context limit]",
            truncated_body.trim_end(),
            char_count,
            max_chars_per_result
        );
        truncated_count += 1;
    }
    truncated_count
}

/// Run a single Ollama request in a new session (no conversation history). Used for SKILL agent.
/// System message = skill content, user message = task. Returns the assistant reply or error string.
pub(crate) async fn run_skill_ollama_session(
    skill_content: &str,
    user_message: &str,
    model_override: Option<String>,
    options_override: Option<crate::ollama::ChatOptions>,
) -> Result<String, String> {
    use tracing::info;
    let messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: skill_content.to_string(),
            images: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
            images: None,
        },
    ];
    info!(
        "Agent router: SKILL session request (user message {} chars)",
        user_message.chars().count()
    );
    let response = send_ollama_chat_messages(messages, model_override, options_override).await?;
    Ok(response.message.content.trim().to_string())
}

/// Run JavaScript via Node.js (if available). Used for RUN_JS in Discord/agent context.
/// Writes code to a temp file and runs `node -e "..."` to eval and print the result.
///
/// **Security:** RUN_JS is agent-triggered and runs with process privileges. Agent or prompt
/// compromise can lead to arbitrary code execution. Treat agent output as untrusted code.
pub(crate) fn run_js_via_node(code: &str) -> Result<String, String> {
    let tmp_dir = crate::config::Config::tmp_js_dir();
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let _ = std::fs::create_dir_all(&tmp_dir);
    let path = tmp_dir.join(format!("mac_stats_js_{}_{}.js", std::process::id(), stamp));
    let path_str = path
        .to_str()
        .ok_or_else(|| "Invalid temp path".to_string())?;

    let mut f = std::fs::File::create(&path).map_err(|e| format!("Create temp file: {}", e))?;
    f.write_all(code.as_bytes())
        .map_err(|e| format!("Write temp file: {}", e))?;
    f.flush().map_err(|e| format!("Flush: {}", e))?;
    drop(f);

    // Node -e script: read file, eval code, print result (no user code in -e, so no escaping).
    let eval_script = r#"const fs=require('fs');const p=process.argv[1];const c=fs.readFileSync(p,'utf8');try{const r=eval(c);console.log(r!==undefined?String(r):'undefined');}catch(e){console.error(e.message);process.exit(1);}"#;
    let out = Command::new("node")
        .arg("-e")
        .arg(eval_script)
        .arg(path_str)
        .output()
        .map_err(|e| format!("Node not available or failed: {}", e))?;

    let _ = std::fs::remove_file(&path);

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(stderr.trim().to_string());
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(stdout.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_at_boundary_returns_full_string_when_short() {
        let body = "hello";
        assert_eq!(truncate_at_boundary(body, 100), "hello");
    }

    #[test]
    fn truncate_at_boundary_exact_length_returns_full_string() {
        let body = "hello";
        assert_eq!(truncate_at_boundary(body, 5), "hello");
    }

    #[test]
    fn truncate_at_boundary_truncates_at_last_word_boundary() {
        let body = "hello world this is a test";
        let result = truncate_at_boundary(body, 11);
        // Last space within first 11 chars is at index 5 → takes "hello " (6 chars)
        assert_eq!(result, "hello ");
    }

    #[test]
    fn truncate_at_boundary_breaks_at_last_space_before_limit() {
        let body = "abcde fghij klmno";
        let result = truncate_at_boundary(body, 10);
        // Last space within first 10 chars is at index 5 → takes "abcde " (6 chars)
        assert_eq!(result, "abcde ");
    }

    #[test]
    fn truncate_at_boundary_uses_later_boundary_when_available() {
        let body = "ab cd ef gh ij kl mn";
        let result = truncate_at_boundary(body, 10);
        // Last space within first 10 chars is at index 8 (before 'g') → takes "ab cd ef " (9 chars)
        assert_eq!(result, "ab cd ef ");
    }

    #[test]
    fn truncate_at_boundary_no_break_point_uses_max() {
        let body = "abcdefghijklmno";
        let result = truncate_at_boundary(body, 5);
        assert_eq!(result, "abcde");
    }

    #[test]
    fn detects_context_overflow_errors() {
        assert!(is_context_overflow_error("Ollama error: context overflow"));
        assert!(is_context_overflow_error(
            "Ollama error: prompt too long for context"
        ));
        assert!(is_context_overflow_error("context length exceeded"));
        assert!(is_context_overflow_error(
            "maximum context length is 4096 tokens"
        ));
        assert!(is_context_overflow_error(
            "exceeds the model's context window"
        ));
        assert!(is_context_overflow_error("request too long for context"));
        assert!(is_context_overflow_error(
            "llama runner: requested more tokens than fit in the context window"
        ));
        assert!(is_context_overflow_error(
            "error: total prompt tokens exceed context size"
        ));
        assert!(is_context_overflow_error("context window exceeded"));
        assert!(is_context_overflow_error("exceeded the context limit"));
        assert!(is_context_overflow_error(
            "error: exceeds the context limit (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: cannot exceed the context limit for this model"
        ));
        assert!(is_context_overflow_error(
            "input exceeds the context window"
        ));
        assert!(is_context_overflow_error(
            "error: prompt does not fit in the context window"
        ));
        assert!(is_context_overflow_error(
            "llama runner: prompt is larger than the context size"
        ));
        assert!(is_context_overflow_error(
            "input falls outside the context window"
        ));
        assert!(is_context_overflow_error(
            "model ran out of context during generation"
        ));
        assert!(is_context_overflow_error(
            "error: context size exceeded (n_ctx=4096)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: exceeded context size for prompt"
        ));
        assert!(is_context_overflow_error(
            "prompt has more tokens than allowed by the model context"
        ));
        assert!(is_context_overflow_error(
            "input exceeds available context; truncate or increase num_ctx"
        ));
        assert!(is_context_overflow_error(
            "the encoded prompt is longer than the context length"
        ));
        assert!(is_context_overflow_error(
            "llama runner: context limit exceeded (max 8192)"
        ));
        assert!(is_context_overflow_error(
            "error: input exceeds context length for this model"
        ));
        assert!(is_context_overflow_error(
            "requested tokens exceed the maximum context window"
        ));
        assert!(is_context_overflow_error(
            "the prompt is too large for the allocated context"
        ));
        assert!(is_context_overflow_error(
            "error: batch cannot fit in context; reduce prompt size"
        ));
        assert!(is_context_overflow_error(
            "model error: prompt does not fit — context is full"
        ));
        assert!(is_context_overflow_error(
            "error: the prompt is too long for this model"
        ));
        assert!(is_context_overflow_error(
            "llama runner: input exceeds maximum context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "maximum context exceeded by encoded prompt"
        ));
        assert!(is_context_overflow_error(
            "insufficient context: increase n_ctx or shorten the prompt"
        ));
        assert!(is_context_overflow_error(
            "error: prompt exceeds the context window size"
        ));
        assert!(is_context_overflow_error(
            "batch unable to fit in context; try a smaller prompt"
        ));
        assert!(is_context_overflow_error(
            "the prompt doesn't fit in the allocated context buffer"
        ));
        assert!(is_context_overflow_error(
            "generation won't fit in remaining context"
        ));
        assert!(is_context_overflow_error(
            "llama runner: encoded prompt is greater than the context length"
        ));
        assert!(is_context_overflow_error(
            "error: n_ctx exceeded — prompt requires more tokens than allocated"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: n_ctx overflow while processing batch"
        ));
        assert!(is_context_overflow_error(
            "model options: num_ctx too small; prompt is longer than num_ctx"
        ));
        assert!(is_context_overflow_error(
            "error: requested num_ctx is larger than the model allows"
        ));
        assert!(is_context_overflow_error(
            "error: exceeds maximum sequence length for this model"
        ));
        assert!(is_context_overflow_error(
            "maximum sequence length exceeded (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: sequence length exceeds n_ctx"
        ));
        assert!(is_context_overflow_error(
            "input sequence is too long for the configured context"
        ));
        assert!(is_context_overflow_error(
            "error: total tokens exceed model limit"
        ));
        assert!(is_context_overflow_error(
            "prompt length exceeds maximum allowed tokens"
        ));
        assert!(is_context_overflow_error(
            "too many tokens in prompt for the context window"
        ));
        assert!(is_context_overflow_error(
            "error: max context exceeded (model limit 4096)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: input exceeds max context for this model"
        ));
        assert!(is_context_overflow_error(
            "generation stopped: prompt extends beyond the context window"
        ));
        assert!(is_context_overflow_error(
            "error: not enough context remaining for completion"
        ));
        assert!(is_context_overflow_error(
            "kv cache error: context buffer overflow during prefill"
        ));
        assert!(is_context_overflow_error(
            "prompt tokens exceed the model's maximum context window"
        ));
        assert!(is_context_overflow_error(
            "input tokens exceed n_ctx; reduce prompt or raise num_ctx"
        ));
        assert!(is_context_overflow_error(
            "error: input exceeds the configured context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: configured context full; shorten the prompt"
        ));
        assert!(is_context_overflow_error(
            "generation would exceed remaining context; aborting"
        ));
        assert!(is_context_overflow_error(
            "error: prompt extends past the context boundary"
        ));
        assert!(is_context_overflow_error(
            "model hit the context limit during prefill"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: reached the context limit (n_ctx)"
        ));
        assert!(is_context_overflow_error(
            "error: encoded batch went over the context limit"
        ));
        assert!(is_context_overflow_error(
            "warning: prompt truncated to fit max context"
        ));
        assert!(is_context_overflow_error(
            "server truncated input due to context length constraints"
        ));
        assert!(is_context_overflow_error(
            "llama runner: context exhausted during prefill"
        ));
        assert!(is_context_overflow_error(
            "error: the model context is fully exhausted; shorten the prompt"
        ));
        assert!(is_context_overflow_error(
            "decode failed: KV cache for this context slot is full"
        ));
        assert!(is_context_overflow_error(
            "insufficient remaining context for the completion request"
        ));
        assert!(is_context_overflow_error(
            "error: encoded prompt exceeds the context size (n_ctx=4096)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: context size exceeds allocated n_ctx"
        ));
        assert!(is_context_overflow_error(
            "error: exceeded the context size for this request"
        ));
        assert!(is_context_overflow_error(
            "model error: the context window is too small for this prompt"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: context capacity exceeded during batch decode"
        ));
        assert!(is_context_overflow_error(
            "error: prompt overflows allocated context buffer"
        ));
        assert!(is_context_overflow_error(
            "prefill failed: input exceeds context slot (n_ctx)"
        ));
        assert!(is_context_overflow_error(
            "error: max_context exceeded by encoded prompt"
        ));
        assert!(is_context_overflow_error(
            "llama runner: prompt is larger than max_context allows"
        ));
        assert!(is_context_overflow_error(
            "validation failed: context_length exceeds the configured maximum"
        ));
        assert!(is_context_overflow_error(
            "json error: context_length too large for model slot"
        ));
        assert!(is_context_overflow_error(
            "API error: maxContext exceeded for this completion request"
        ));
        assert!(is_context_overflow_error(
            "validation: contextLength exceeds server maximum (8192)"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: n_ctx_per_seq too small for encoded prompt"
        ));
        assert!(is_context_overflow_error(
            "error: exceeds the model context limit (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "API error: exceeded the model context for this request"
        ));
        assert!(is_context_overflow_error(
            "validation: model context exceeded by encoded prompt"
        ));
        assert!(is_context_overflow_error(
            "json: context_window exceeds the configured maximum"
        ));
        assert!(is_context_overflow_error(
            "options: context_window too small for prompt"
        ));
        assert!(is_context_overflow_error(
            "validation: context_limit exceeds the configured maximum"
        ));
        assert!(is_context_overflow_error(
            "json error: contextlimit overflow for this request"
        ));
        assert!(is_context_overflow_error(
            "error: cannot exceed the maximum context for this model"
        ));
        assert!(is_context_overflow_error(
            "llama runner: exceeded maximum context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "model error: maximum context is exceeded by encoded prompt"
        ));
        assert!(is_context_overflow_error(
            "API error: context token limit exceeded (8192)"
        ));
        assert!(is_context_overflow_error(
            "error: encoded prompt exceeds the context token limit"
        ));
        assert!(is_context_overflow_error(
            "server: exceeded the context token limit for slot 0"
        ));
        assert!(is_context_overflow_error(
            "error: encoded prompt exceeds the model's maximum context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: input exceeded the model's maximum context for this model"
        ));
        assert!(is_context_overflow_error(
            "validation: batch would exceed the model's maximum context"
        ));
        assert!(is_context_overflow_error(
            "error: prompt extends beyond the context window"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: generation ran over the context window boundary"
        ));
        assert!(is_context_overflow_error(
            "error: tokens fall outside of the context window"
        ));
        assert!(is_context_overflow_error(
            "llama runner: prompt extends beyond context window (8192)"
        ));
        assert!(is_context_overflow_error(
            "API: encoded span runs over context window for slot 0"
        ));
        assert!(is_context_overflow_error(
            "error: batch indices fall outside context window"
        ));
        assert!(is_context_overflow_error(
            "validator: token range lies outside of context window"
        ));
        assert!(is_context_overflow_error(
            "error: input exceeds model maximum context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "llama runner: cannot exceed the allowed maximum context for this slot"
        ));
        assert!(is_context_overflow_error(
            "API error: encoded prompt is too large for max context"
        ));
        assert!(is_context_overflow_error(
            "llama runner: running out of context during decode"
        ));
        assert!(is_context_overflow_error(
            "error: context budget exceeded for this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: conversation is too long for the model context window"
        ));
        assert!(is_context_overflow_error(
            "openai error: invalid_request_error (context_length_exceeded)"
        ));
        assert!(is_context_overflow_error(
            "{\"error\":{\"code\":\"max_context_length_exceeded\",\"message\":\"...\"}}"
        ));
        assert!(is_context_overflow_error(
            "gateway: type invalid_request_error code context_window_exceeded"
        ));
        assert!(is_context_overflow_error(
            "llama.cpp: inference failed: max_context_exceeded"
        ));
        assert!(is_context_overflow_error(
            "error: exceeds this model's context (8192 tokens)"
        ));
        assert!(is_context_overflow_error(
            "API: exceeded this model's context window during prefill"
        ));
        assert!(is_context_overflow_error(
            "validation: cannot exceed this model's context limit"
        ));
        assert!(is_context_overflow_error(
            "API error: messages exceed the model's context window"
        ));
        assert!(is_context_overflow_error(
            "openai: messages exceeded maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: messages exceed available context on this request"
        ));
        assert!(is_context_overflow_error(
            "API error: message exceeds the model's context window"
        ));
        assert!(is_context_overflow_error(
            "validation: your message exceeded maximum context for this model"
        ));
        assert!(is_context_overflow_error(
            "API: the message is too long for the model's context window"
        ));
        assert!(is_context_overflow_error(
            "error: messages are too long for maximum context on this model"
        ));
        assert!(is_context_overflow_error(
            "batch: inputs are too long for the model's context window"
        ));
        assert!(is_context_overflow_error(
            "API: inputs were too long; exceeded available context on this request"
        ));
        assert!(is_context_overflow_error(
            "validation: input is too long for context length 8192"
        ));
        assert!(is_context_overflow_error(
            "request failed: message was too long; exceeds available context"
        ));
        assert!(is_context_overflow_error(
            "error: request exceeds your maximum allowed context for this model"
        ));
        assert!(is_context_overflow_error(
            "validation: cannot exceed max allowed context on this endpoint"
        ));
        assert!(is_context_overflow_error(
            "gateway: shard overflow relative to maximum context on this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: context length limit exceeded for the chat completion"
        ));
        assert!(is_context_overflow_error(
            "openai error: invalid_request_error (context_budget_exceeded)"
        ));
        assert!(is_context_overflow_error(
            "API error: inputs exceed the model's context window"
        ));
        assert!(is_context_overflow_error(
            "gateway: inputs exceeded available context on this request"
        ));
        assert!(is_context_overflow_error(
            "validation: inputs exceed context length for this model"
        ));
        assert!(is_context_overflow_error(
            "API: input exceeded available context on this request"
        ));
        assert!(is_context_overflow_error(
            "validation: input exceed the model's context window"
        ));
        assert!(is_context_overflow_error(
            "your message is too long for this model's context"
        ));
        assert!(is_context_overflow_error(
            "chat: messages exceed the model's context on this turn"
        ));
        assert!(is_context_overflow_error(
            "batch: inputs exceeded the model's context"
        ));
        assert!(is_context_overflow_error(
            "API: contents exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: contents exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: content exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "error: content exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: outputs exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: outputs exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: output exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "runner: output exceeded the context window during decode"
        ));
        assert!(is_context_overflow_error(
            "API: responses exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: responses exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: response exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: response exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: requests exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: requests exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: request exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: request exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: queries exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: queries exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: query exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: query exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: calls exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: calls exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: call exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: call exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: batches exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: batches exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: batch exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: batch exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: tokens exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: tokens exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: token exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: token exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: items exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: items exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: item exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: item exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: entries exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: entries exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: entry exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: entry exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: records exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: records exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: record exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: record exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: chunks exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: chunks exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: chunk exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: chunk exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: documents exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: documents exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: document exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: document exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: files exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: files exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: file exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: file exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: lines exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: lines exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: line exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: line exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: cells exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: cells exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: cell exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: cell exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: rows exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: rows exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: row exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: row exceeded the context window"
        ));
        assert!(is_context_overflow_error(
            "API: columns exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: columns exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: column exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: column exceeded the context window"
        ));
    }

    /// FEAT-D391: the `message`/`inputs` … `too long` + explicit-slot conjunct uses
    /// `contains_phrase_after_ident_boundary`, not substring `contains`, on each phrase.
    #[test]
    fn message_input_too_long_phrases_use_ident_boundary_on_conjunct() {
        let s1 = "fixture: submessage is too long (pipeline note)";
        let l1 = s1.to_lowercase();
        assert!(l1.contains("message is too long"));
        assert!(!contains_phrase_after_ident_boundary(&l1, "message is too long"));

        let l2 = "micromessage is too long".to_lowercase();
        assert!(l2.contains("message is too long"));
        assert!(!contains_phrase_after_ident_boundary(&l2, "message is too long"));

        let l3 = "metainputs are too long".to_lowercase();
        assert!(l3.contains("inputs are too long"));
        assert!(!contains_phrase_after_ident_boundary(&l3, "inputs are too long"));

        assert!(!is_context_overflow_error(s1));
    }

    #[test]
    fn does_not_match_unrelated_errors() {
        assert!(!is_context_overflow_error(
            "Ollama HTTP 503: service unavailable"
        ));
        assert!(!is_context_overflow_error("connection refused"));
        assert!(!is_context_overflow_error("rate limit exceeded"));
        assert!(!is_context_overflow_error("timeout"));
        assert!(!is_context_overflow_error(
            "cannot fit model weights into GPU memory"
        ));
        assert!(!is_context_overflow_error("request too large"));
        assert!(!is_context_overflow_error("unable to fit in GPU memory"));
        assert!(!is_context_overflow_error("the file won't fit on disk"));
        assert!(!is_context_overflow_error(
            "diagnostic: n_ctx=8192 num_ctx=8192 (ok)"
        ));
        assert!(!is_context_overflow_error(
            "hint: set num_ctx in Modelfile to match the model card"
        ));
        assert!(!is_context_overflow_error(
            "too many tokens in request (rate limited)"
        ));
        assert!(!is_context_overflow_error(
            "billing: total tokens exceed your monthly quota"
        ));
        assert!(!is_context_overflow_error(
            "API error: prompt tokens exceed per-minute billing cap"
        ));
        assert!(!is_context_overflow_error(
            "validation: contents exceed max attachment size (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "batch: contents exceed per-request rate limits"
        ));
        assert!(!is_context_overflow_error(
            "GPU: shader outputs exceed max attachment slots (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "pipeline: outputs exceed per-stage rate limits"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: responses exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: requests exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "config: microrequests exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metarequests exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subrequest exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prerequest exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: microqueries exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaqueries exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subquery exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prequery exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: requery exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "bioinfo: subsequence length exceeds alignment window (not llm context)"
        ));
        assert!(!is_context_overflow_error(
            "kernel: microexceeds maximum sequence length in fused op (not context)"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromaximum sequence length exceeded in synthetic fixture"
        ));
        assert!(!is_context_overflow_error(
            "parser: microinput sequence is too long for amino-acid field (pipeline limit)"
        ));
        assert!(!is_context_overflow_error(
            "templating: microprompt length exceeds macro substitution depth (not context)"
        ));
        assert!(!is_context_overflow_error(
            "forms: microprompt too long for field schema (pipeline limit)"
        ));
        assert!(!is_context_overflow_error(
            "macros: microprompt is too long for expansion buffer (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "templates: microprompt has more tokens than the lexer allows (not model window)"
        ));
        assert!(!is_context_overflow_error(
            "ui: microprompt exceeds the context menu width (layout)"
        ));
        assert!(!is_context_overflow_error(
            "metrics: microcontext overflow counter reset (unrelated)"
        ));
        assert!(!is_context_overflow_error(
            "parser: microcontext length exceeded in field width (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromaximum context length in synthetic fixture (not model)"
        ));
        assert!(!is_context_overflow_error(
            "fixture: microexceeds the model's context window in stress test (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "stats: microexceeds the model context in batch column (telemetry)"
        ));
        assert!(!is_context_overflow_error(
            "log: microexceeded the model context in rollup row (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "cli: microexceed the model context is not a valid flag (typo)"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromodel context exceeded in fixture name (not ollama)"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromodel context limit was exceeded in synthetic metric (not ollama)"
        ));
        assert!(!is_context_overflow_error(
            "fixture: microexceeds the context window in UI layout test (not tokens)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microcontext window exceeded in accessibility tree (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "billing: microexceeded the context limit is not an overflow message"
        ));
        assert!(!is_context_overflow_error(
            "billing: microexceeds the context limit is not an overflow message"
        ));
        assert!(!is_context_overflow_error(
            "billing: microexceed the context limit is not an overflow message"
        ));
        assert!(!is_context_overflow_error(
            "lint: microrequested more tokens than column width in fixture (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "ui: microfit in the context panel is a layout hint (not overflow)"
        ));
        assert!(!is_context_overflow_error(
            "layout: microlarger than the context sidebar (not model window)"
        ));
        assert!(!is_context_overflow_error(
            "a11y: microoutside the context window in devtools tree (not tokens)"
        ));
        assert!(!is_context_overflow_error(
            "a11y: microoutside of the context window in fixture (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microbeyond the context window in CSS var name (not overflow)"
        ));
        assert!(!is_context_overflow_error(
            "layout: microbeyond the context panel width (not model window)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microover the context window in test label (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "css: microbeyond context window token in stylesheet (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "css: microover context window in shorthand (not overflow)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microoutside context window in selector (not model)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microoutside of context window in snapshot (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "metrics: microran out of context handles (unrelated)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microrunning out of context switches in parser (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microcontext size exceeded column width in fixture (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "stats: microexceeded context size metric is a dashboard label (not tokens)"
        ));
        assert!(!is_context_overflow_error(
            "ui: microexceeds available context panel width (layout)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microcontext limit exceeded in CSS var (not model)"
        ));
        assert!(!is_context_overflow_error(
            "layout: microexceeds context length in flex line (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "gpu: microinsufficient context for draw call (not llm window)"
        ));
        assert!(!is_context_overflow_error(
            "layout: micronot enough context in grid track (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "ui: micropast the context sidebar edge (not overflow)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microreached the context limit in test harness (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "metrics: microhit the context limit counter label (unrelated)"
        ));
        assert!(!is_context_overflow_error(
            "layout: microover the context limit line in diagram (not tokens)"
        ));
        assert!(!is_context_overflow_error(
            "parser: microcontext exhausted the small-string buffer (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "ui: request too long for microcontext label (layout)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microconversation is too long for variable name (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "log: truncated microcontext window in debug view (not tokens)"
        ));
        assert!(!is_context_overflow_error(
            "gpu: kv cache is full but microcontext binding failed (not llm window)"
        ));
        assert!(!is_context_overflow_error(
            "layout: prefill microcontext panel exceeds width (not model)"
        ));
        assert!(!is_context_overflow_error(
            "scheduler: microinsufficient remaining context switches (os)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microexceeds the context size hint in UI mock (not model)"
        ));
        assert!(!is_context_overflow_error(
            "stats: microcontext size exceeds column header in sheet (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "log: microexceeded the context size in rollup (telemetry)"
        ));
        assert!(!is_context_overflow_error(
            "api: microcontext token limit exceeded rate (unrelated)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microexceeds the context token limit label in fixture (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "log: microexceeded the context token limit in column name (not ollama)"
        ));
        assert!(!is_context_overflow_error(
            "lint: microexceeds this model's context field in schema (not window)"
        ));
        assert!(!is_context_overflow_error(
            "fixture: microexceeded this model's context row in CSV (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "cli: microexceed this model's context flag typo (not overflow)"
        ));
        assert!(!is_context_overflow_error(
            "config: micromessages exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metamessages exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: submessage exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: premessage exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: remessage exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: messages exceed UI bounds (microcontext window width)"
        ));
        assert!(!is_context_overflow_error(
            "billing: total tokens exceed microcontext row counter (telemetry)"
        ));
        assert!(!is_context_overflow_error(
            "API: prompt tokens exceed microwindow title length (not llm)"
        ));
        assert!(!is_context_overflow_error(
            "config: microinputs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metainputs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subinput exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preinput exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: reinput exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: microcontents exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metacontents exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subcontent exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: precontent exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: recontent exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: microoutputs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaoutputs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: suboutput exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preoutput exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: reoutput exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: microresponses exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaresponses exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subresponse exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preresponse exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: reresponse exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: micrototal prompt tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: microrequested tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: micrototal tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metatotal tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subprompt tokens exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preprompt tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subinput tokens exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preinput tokens exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "http: request exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: queries exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: calls exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: batches exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: tokens exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: items exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: entries exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: records exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: chunks exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: documents exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: files exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "config: microcalls exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metabatches exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subcall exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: precall exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "queue: recall exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "config: microitems exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaentries exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subentry exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microchunks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metadocuments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subdocument exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microfiles exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metelines exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "routing: subline exceed header cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microcells exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromaximum context exceeded in synthetic metric (not ollama)"
        ));
        assert!(!is_context_overflow_error(
            "lint: micromax context row in dashboard grid (not llm window)"
        ));
        assert!(!is_context_overflow_error(
            "stats: microcontext length limit exceeded in column name (telemetry)"
        ));
        assert!(!is_context_overflow_error(
            "billing: tokens exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: items exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: entries exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: records exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: chunks exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: documents exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "billing: files exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: file exceed max upload size (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: lines exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: lines exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "log: line exceed max line length (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: cells exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: cells exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "spreadsheet: cell exceed max formula length (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: rows exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: rows exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "spreadsheet: row exceed max row height (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: columns exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: columns exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "spreadsheet: column exceed max column width (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: tables exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: tables exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: table exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: table exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: tables exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: tables exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "SQL: constables exceed department headcount (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "layout: stable exceed viewport width (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: blocks exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: blocks exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: block exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: block exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: blocks exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: blocks exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: block exceed max object size (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "traffic: roadblocks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "safety: roadblock exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "skincare: sunblock exceed SPF labeling limits (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: segments exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: segments exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: segment exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: segment exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: segments exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: segments exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "video: segment exceed max GOP duration (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: microsegments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: multisegment exceeds max track width (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: sections exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: sections exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: section exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: section exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: sections exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: sections exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "layout: section exceed max heading depth (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "docs: subsections exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "geometry: intersection exceed tolerance (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: paragraphs exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: paragraphs exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: paragraph exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: paragraph exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: paragraphs exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: paragraphs exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "layout: paragraph exceed max width in twips (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "legal: counterparagraphs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "brief: counterparagraph exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: sentences exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: sentences exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: sentence exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: sentence exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: sentences exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: sentences exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "NLP: sentence exceed max tokens per span (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: microsentences exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: microsentence exceed display width (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: words exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: words exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: word exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: word exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: words exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: words exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "NLP: word exceed max syllables per token (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "lexicon: buzzwords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "SEO: keywords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "puzzles: crosswords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "RAG: microwords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "glossary: buzzword exceed display width (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: characters exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: characters exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: character exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: character exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: characters exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: characters exceeded daily usage cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "UI: character exceed max field width in pixels (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: megacharacters exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "regex: metacharacters exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: noncharacter exceed token class limit (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: strings exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: strings exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: string exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: string exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: strings exceed per-field length limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: strings exceeded daily ingest cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "codec: string exceed max UTF-8 width on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "parser: substring exceed lexer token budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microstrings exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metastrings exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: arrays exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: arrays exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: array exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: array exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: arrays exceed per-batch element limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: arrays exceeded daily request cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: array exceed max nesting depth on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microarrays exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaarrays exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subarray exceed lexer recursion budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: objects exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: objects exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: object exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: object exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: objects exceed per-request reference limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: objects exceeded daily create cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: object exceed max property count on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microobjects exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaobjects exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subobject exceed deserialization budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: elements exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: elements exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: element exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: element exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: elements exceed per-batch cardinality limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: elements exceeded daily ingest cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: element exceed max tensor rank on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microelements exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaelements exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subelement exceed nesting budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: nodes exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: nodes exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: node exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: node exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: nodes exceed per-cluster membership limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: nodes exceeded daily provision cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: node exceed max depth on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micronodes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metanodes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subnode exceed traversal budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: edges exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: edges exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: edge exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: edge exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: edges exceed per-graph fan-out limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: edges exceeded daily relationship cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: edge exceed max adjacency on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microedges exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaedges exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subedge exceed traversal budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "wedge exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: vertices exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: vertices exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: vertex exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: vertex exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: vertices exceed per-mesh topology limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: vertices exceeded daily geometry cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: vertex exceed max valence on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microvertices exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metavertices exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subvertex exceed fan-out budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supervertex exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: faces exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: faces exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: face exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: face exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: faces exceed per-mesh polygon limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: faces exceeded daily geometry cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: face exceed max facet count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microfaces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metafaces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subface exceed facet budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "surface exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: triangles exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: triangles exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: triangle exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: triangle exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: triangles exceed per-mesh primitive limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: triangles exceeded daily geometry cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: triangle exceed max strip count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microtriangles exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metatriangles exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subtriangle exceed strip budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supertriangle exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: polygons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: polygons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: polygon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: polygon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: polygons exceed per-layer ring limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: polygons exceeded daily geometry cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: polygon exceed max hole count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micropolygons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metapolygons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subpolygon exceed ring budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superpolygon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: meshes exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: meshes exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: mesh exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: mesh exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: meshes exceed per-draw mesh limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: meshes exceeded daily geometry cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: mesh exceed max submesh count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micromeshes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metameshes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: submesh exceed LOD budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supermesh exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: voxels exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: voxels exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: voxel exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: voxel exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: voxels exceed per-chunk grid limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: voxels exceeded daily volume cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: voxel exceed max brick count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microvoxels exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metavoxels exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subvoxel exceed LOD budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supervoxel exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: particles exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: particles exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: particle exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: particle exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: particles exceed per-emitter rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: particles exceeded daily simulation cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: particle exceed max burst count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microparticles exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaparticles exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subparticle exceed LOD budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superparticle exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: molecules exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: molecules exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: molecule exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: molecule exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: molecules exceed per-reaction rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: molecules exceeded daily structure cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: molecule exceed max bond count on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micromolecules exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metamolecules exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: submolecule exceed valence budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supermolecule exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: atoms exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: atoms exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: atom exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: atom exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: atoms exceed per-basis rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: atoms exceeded daily structure cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: atom exceed max valence on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microatoms exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaatoms exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subatom exceed shell budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superatom exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: ions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: ions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: ion exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: ion exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: ions exceed per-species rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: ions exceeded daily plasma cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: ion exceed max charge on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subion exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superion exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: million exceed per-client rate limits for this endpoint"
        ));
        assert!(is_context_overflow_error(
            "API: electrons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: electrons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: electron exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: electron exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: electrons exceed per-beam rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: electrons exceeded daily cathode cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: electron exceed max spin slots on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microelectrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaelectrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subelectron exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superelectron exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: protons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: protons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: proton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: proton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: protons exceed per-beam rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: protons exceeded daily nucleus cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: proton exceed max charge states on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microprotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaprotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subproton exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superproton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: neutrons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: neutrons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: neutron exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: neutron exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: neutrons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: neutrons exceeded daily cross-section cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: neutron exceed max scattering channels on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microneutrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaneutrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subneutron exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superneutron exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: quarks exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: quarks exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: quark exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: quark exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: quarks exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: quarks exceeded daily flavor cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: quark exceed max Wilson lines on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microquarks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaquarks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subquark exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superquark exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: gluons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: gluons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: gluon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: gluon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: gluons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: gluons exceeded daily color-octet cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: gluon exceed max gauge links on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microgluons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metagluons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subgluon exceed mobility budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supergluon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: bosons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: bosons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: boson exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: boson exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: bosons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: bosons exceeded daily mode-occupancy cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: boson exceed max virtual lines on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microbosons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metabosons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subboson exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superboson exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: leptons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: leptons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: lepton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: lepton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: leptons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: leptons exceeded daily family cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: lepton exceed max weak vertices on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microleptons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaleptons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: sublepton exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superlepton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: hadrons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: hadrons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: hadron exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: hadron exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: hadrons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: hadrons exceeded daily jet-multiplicity cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: hadron exceed max fragmentation on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microhadrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metahadrons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subhadron exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superhadron exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: photons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: photons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: photon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: photon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: photons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: photons exceeded daily fluence cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: photon exceed max beam occupancy on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microphotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaphotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subphoton exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superphoton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: phonons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: phonons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: phonon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: phonon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: phonons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: phonons exceeded daily branch-occupancy cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: phonon exceed max mode occupancy on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microphonons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaphonons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subphonon exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superphonon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: excitons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: excitons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: exciton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: exciton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: excitons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: excitons exceeded daily well-occupancy cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: exciton exceed max pair density on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microexcitons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaexcitons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subexciton exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superexciton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: polarons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: polarons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: polaron exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: polaron exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: polarons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: polarons exceeded daily lattice-deformation cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: polaron exceed max coupling budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micropolarons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metapolarons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subpolaron exceed deformation budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superpolaron exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: plasmons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: plasmons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: plasmon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: plasmon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: plasmons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: plasmons exceeded daily near-field mode cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: plasmon exceed max dispersion budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microplasmons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaplasmons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subplasmon exceed coupling budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superplasmon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: solitons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: solitons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: soliton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: soliton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: solitons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: solitons exceeded daily nonlinear-mode cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: soliton exceed max pulse budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microsolitons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metasolitons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subsoliton exceed dispersion budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supersoliton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: instantons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: instantons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: instanton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: instanton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: instantons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: instantons exceeded daily gauge-orbit cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: instanton exceed max tunneling budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microinstantons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metainstantons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subinstanton exceed sector budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superinstanton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: skyrmions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: skyrmions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: skyrmion exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: skyrmion exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: skyrmions exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: skyrmions exceeded daily track cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: skyrmion exceed max topological-charge budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microskyrmions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaskyrmions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subskyrmion exceed winding budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superskyrmion exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: magnons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: magnons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: magnon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: magnon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: magnons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: magnons exceeded daily spin-wave cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: magnon exceed max k-space budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micromagnons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metamagnons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: submagnon exceed branch budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supermagnon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: rotons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: rotons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: roton exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: roton exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: rotons exceed per-target rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: rotons exceeded daily Landau-level cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: roton exceed max dispersion budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microrotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metarotons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subroton exceed branch budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superroton exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: anyons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: anyons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: anyon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: anyon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: anyons exceed per-braid rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: anyons exceeded daily fusion-channel cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: anyon exceed max braiding budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microanyons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaanyons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subanyon exceed charge budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superanyon exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "geology: canyon exceed maximum depth on this survey (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: fluxons exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: fluxons exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: fluxon exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: fluxon exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: fluxons exceed per-array rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: fluxons exceeded daily SQUID cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: fluxon exceed max shunt budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microfluxons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metafluxons exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subfluxon exceed bias budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "superfluxon exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: vortices exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: vortices exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: vortex exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: vortex exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: vortices exceed per-mesh rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: vortices exceeded daily circulation cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: vortex exceed max swirl budget on this graph field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microvortices exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metavortices exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "parser: subvortex exceed core budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "supervortex exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "API: bytes exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: bytes exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: byte exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: byte exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: bytes exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: bytes exceeded daily upload cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: byte exceed max object size on this bucket (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: megabytes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "transfer: kilobytes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "codec: kilobyte exceed frame size cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: bits exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: bits exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: bit exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: bit exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: bits exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: bits exceeded daily transfer cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "codec: bit exceed max frame size on this stream (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: megabits exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "transfer: kilobits exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "codec: kilobit exceed symbol size cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "parser: rabbit exceed max nesting depth (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: fields exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: fields exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: field exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: field exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: fields exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: fields exceeded daily write cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: field exceed max string length on this column (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "RAG: battlefields exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "geo: cornfields exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: afield exceed display width cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: subfield exceed nesting depth (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: values exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: values exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: value exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: value exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: values exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: values exceeded daily write cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: value exceed max numeric range on this column (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "stats: eigenvalues exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "solver: meanvalues exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "finance: devalue exceed policy threshold (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "pricing: overvalue exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: keys exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: keys exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: key exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: key exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: keys exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: keys exceeded daily write cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: key exceed max name length on this column (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "UI: hotkeys exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "property: turnkeys exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "zoo: monkey exceed feeding schedule cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "auth: passkey exceed device binding limit (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: properties exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: properties exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: property exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: property exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: properties exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: properties exceeded daily write cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "schema: property exceed max nesting depth on this object (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microproperties exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subproperty exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: schemas exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: schemas exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: schema exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: schema exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: schemas exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: schemas exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "registry: schema exceed max $ref depth on this object (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microschemas exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subschema exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: parameters exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: parameters exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: parameter exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: parameter exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: parameters exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: parameters exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "openapi: parameter exceed max in-path segments on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microparameters exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaparameters exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subparameter exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "API: arguments exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: arguments exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: argument exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: argument exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: arguments exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: arguments exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "graphql: argument exceed max list depth on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microarguments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaarguments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subargument exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "GraphQL: variables exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: variables exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: variable exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "gateway: variable exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: variables exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: variables exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "template: variable exceed max substitution depth on this field (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: metavariables exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: hypervariables exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: multivariable exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "layout: subvariable exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: headers exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: headers exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: header exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: header exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: headers exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: headers exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: header exceed max allowed total size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microheaders exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaheaders exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subheader exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: cookies exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: cookies exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: cookie exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: cookie exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: cookies exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: cookies exceeded daily compile cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: cookie exceed max allowed total count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microcookies exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metacookies exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subcookie exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: bodies exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: bodies exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: body exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: body exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: bodies exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: bodies exceeded max upload size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: body exceed max allowed total size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microbodies exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metabodies exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subbody exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: parts exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: parts exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: part exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: part exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: parts exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: parts exceeded max multipart count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: part exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microparts exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaparts exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subpart exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: pieces exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: pieces exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: piece exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: piece exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: pieces exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: pieces exceeded max puzzle count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: piece exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micropieces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metapieces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subpiece exceed display cap (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: shards exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: shards exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: shard exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: shard exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: shards exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: shards exceeded max replica count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: shard exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microshards exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metashards exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subshard exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: reshard exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: fragments exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: fragments exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: fragment exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: fragment exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: fragments exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: fragments exceeded max packet count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: fragment exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microfragments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metafragments exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subfragment exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: refragment exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: packets exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: packets exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: packet exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: packet exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: packets exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: packets exceeded max MTU count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: packet exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micropackets exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metapackets exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subpacket exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: repacket exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: frames exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: frames exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: frame exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: frame exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: frames exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: frames exceeded max GOP size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: frame exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microframes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaframes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subframe exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: reframe exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: samples exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: samples exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: sample exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: sample exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: samples exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: samples exceeded max batch size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: sample exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microsamples exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metasamples exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "dsp: subsample exceed decimation cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: resample exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: observations exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: observations exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: observation exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: observation exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: observations exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: observations exceeded max batch size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: observation exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microobservations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaobservations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subobservation exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preobservation exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: events exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: events exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: event exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: event exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: events exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: events exceeded max batch size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: event exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microevents exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaevents exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subevent exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preevent exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "workflow: prevent exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: traces exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: traces exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: trace exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: trace exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: traces exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: traces exceeded max span count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: trace exceed max allowed depth on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microtraces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metatraces exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: subtrace exceed span cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: pretrace exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: retrace exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: spans exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: spans exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: span exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: span exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: spans exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: spans exceeded max attribute count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: span exceed max allowed depth on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microspans exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaspans exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: subspan exceed link cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prespan exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: respan exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: attributes exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: attributes exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: attribute exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: attribute exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: attributes exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: attributes exceeded max key count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: attribute exceed max allowed size on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microattributes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaattributes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: subattribute exceed tag cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preattribute exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: reattribute exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: links exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: links exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: link exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: link exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: links exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: links exceeded max per-span cap on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: link exceed max allowed count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microlinks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metalinks exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: sublink exceed span cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prelink exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: relink exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: scopes exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: scopes exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: scope exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: scope exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: scopes exceed per-client rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: scopes exceeded max instrumentation cap on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: scope exceed max allowed depth on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microscopes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metascopes exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: subscope exceed instrumentation cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prescope exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: rescope exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: resources exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: resources exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: resource exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: resource exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: resources exceed per-tenant rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: resources exceeded max attribute bytes on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: resource exceed max allowed labels on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microresources exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaresources exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: subresource exceed descriptor cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preresource exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: reresource exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: metrics exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: metrics exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: metric exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: metric exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: metrics exceed per-tenant rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: metrics exceeded max series cardinality on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: metric exceed max label count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micrometrics exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metametrics exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "otel: submetric exceed descriptor cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: premetric exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: remetric exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: dimensions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: dimensions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: dimension exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: dimension exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: dimensions exceed per-request tensor rank limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: dimensions exceeded max axis count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: dimension exceed max embedding width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microdimensions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metadimensions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subdimension exceed display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: predimension exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: redimension exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: tensors exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: tensors exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: tensor exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: tensor exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: tensors exceed per-request graph node limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: tensors exceeded max operand count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: tensor exceed max rank on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microtensors exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metatensors exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subtensor exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: pretensor exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: retensor exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: activations exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: activations exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: activation exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: activation exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: activations exceed per-layer buffer limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: activations exceeded max feature map count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: activation exceed max channel width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microactivations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaactivations exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subactivation exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preactivation exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: reactivation exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: gradients exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: gradients exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: gradient exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: gradient exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: gradients exceed per-step clip limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: gradients exceeded max backward-pass depth on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: gradient exceed max Jacobian rows on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microgradients exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metagradients exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subgradient exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: pregradient exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: regradient exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: weights exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: weights exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: weight exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: weight exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: weights exceed per-layer L2 clip limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: weights exceeded max trainable parameter count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: weight exceed max shard rows on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microweights exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaweights exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subweight exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preweight exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: reweight exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: biases exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: biases exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: bias exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: bias exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: biases exceed per-layer offset limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: biases exceeded max initializer count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: bias exceed max row width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microbiases exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metabiases exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subbias exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prebias exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: rebias exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: layers exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: layers exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: layer exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: layer exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: layers exceed per-stack depth limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: layers exceeded max module count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: layer exceed max channel width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microlayers exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metalayers exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: sublayer exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prelayer exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: relayer exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: heads exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: heads exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: head exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: head exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: heads exceed per-attention fan-in limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: heads exceeded max parallel workers on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: head exceed max replica count on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microheads exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaheads exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subhead exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prehead exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "debug: rehead exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "gateway: positions exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: positions exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: position exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: position exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: positions exceed max sequence index for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: positions exceeded allowed KV slots on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: position exceed max slot width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micropositions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metapositions exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "grammar: subposition exceed rule depth (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preposition exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: reposition exceed grid bounds (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: embeddings exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: embeddings exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: embedding exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: embedding exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: embeddings exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: embeddings exceeded max vector dimensions on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: embedding exceed max batch width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microembeddings exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaembeddings exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subembedding exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preembedding exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "index: reembedding exceed cache budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: logits exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: logits exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: logit exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: logit exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: logits exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: logits exceeded max vocabulary width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: logit exceed max batch dim on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micrologits exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metalogits exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: sublogit exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prelogit exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "index: relogit exceed cache budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: probabilities exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: probabilities exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: probability exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: probability exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: probabilities exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: probabilities exceeded max softmax width on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: probability exceed max sampling dim on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microprobabilities exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metaprobabilities exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: subprobability exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: preprobability exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "index: reprobability exceed cache budget (no model context configured)"
        ));
        assert!(is_context_overflow_error(
            "gateway: logprobs exceed the model's context window on this request"
        ));
        assert!(is_context_overflow_error(
            "batch: logprobs exceeded available context for the completion"
        ));
        assert!(is_context_overflow_error(
            "validation: logprob exceed maximum context length for this model"
        ));
        assert!(is_context_overflow_error(
            "proxy: logprob exceeded the context window"
        ));
        assert!(!is_context_overflow_error(
            "HTTP: logprobs exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "billing: logprobs exceeded max return-n on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "http: logprob exceed max top-k on this route (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: micrologprobs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metalogprobs exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "graph: sublogprob exceed op cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: prelogprob exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "index: relogprob exceed cache budget (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "config: microrecords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "tuning: metarecords exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "layout: subrecord exceed row cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: rerecord exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "pipeline: prerecord exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "microcolumns exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "multicolumn exceeds max column width (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "arrows exceed the model's context window on this request"
        ));
        assert!(!is_context_overflow_error(
            "throws exceed available context for the completion"
        ));
        assert!(!is_context_overflow_error(
            "arrow exceed per-row display cap (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "catalog: entry exceed max sku length (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "audit: record exceed max field length (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "storage: document exceed max attachment size (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "database: queries exceeded the allowed execution time"
        ));
        assert!(!is_context_overflow_error(
            "survey requests exceed the allowed quota for this form"
        ));
        assert!(!is_context_overflow_error(
            "survey responses exceed the allowed quota for this form"
        ));
        assert!(!is_context_overflow_error(
            "network buffer full; retry later"
        ));
        assert!(!is_context_overflow_error(
            "response truncated for display (unrelated to model context)"
        ));
        assert!(!is_context_overflow_error(
            "log truncated; see full trace for context"
        ));
        assert!(!is_context_overflow_error(
            "this feature is fully supported in the application context"
        ));
        assert!(!is_context_overflow_error(
            "gpu kv cache is full (tensor allocation failed)"
        ));
        assert!(!is_context_overflow_error(
            "prefill completed; context tensors initialized"
        ));
        assert!(!is_context_overflow_error(
            "Modelfile: allocated context is 8192 tokens (default)"
        ));
        assert!(!is_context_overflow_error(
            "options: max_context=8192 num_ctx=8192 (ok)"
        ));
        assert!(!is_context_overflow_error(
            "request fields: context_length, temperature, stream"
        ));
        assert!(!is_context_overflow_error(
            "config: maxContext=8192 num_ctx=8192 (startup ok)"
        ));
        assert!(!is_context_overflow_error(
            "docs: contextLength optional in request schema"
        ));
        assert!(!is_context_overflow_error(
            "diagnostic: n_ctx_per_seq=4096 batch ok"
        ));
        assert!(!is_context_overflow_error(
            "defaults: context_window=8192 num_ctx=8192 (ok)"
        ));
        assert!(!is_context_overflow_error(
            "docs: context_window optional in request schema"
        ));
        assert!(!is_context_overflow_error(
            "note: tune model context for your workload (no error)"
        ));
        assert!(!is_context_overflow_error(
            "options: context_limit=8192 num_ctx=8192 (ok)"
        ));
        assert!(!is_context_overflow_error(
            "request fields: context_limit, temperature, stream"
        ));
        assert!(!is_context_overflow_error(
            "config: contextLimit=8192 (startup ok)"
        ));
        assert!(!is_context_overflow_error(
            "API token limit exceeded: upgrade your plan"
        ));
        assert!(!is_context_overflow_error(
            "docs: default maximum context is 8192 tokens (informational)"
        ));
        assert!(!is_context_overflow_error(
            "hint: tune max context in Modelfile for longer threads (no error)"
        ));
        assert!(!is_context_overflow_error(
            "running out of patience while waiting for the API"
        ));
        assert!(!is_context_overflow_error(
            "note: context budget is 1M tokens on the enterprise plan (informational)"
        ));
        assert!(!is_context_overflow_error(
            "UI: conversation is too long to display; click to expand"
        ));
        assert!(!is_context_overflow_error(
            "migration: dropped column old_context_length_exceeded from analytics"
        ));
        assert!(!is_context_overflow_error(
            "refactor: rename old_max_context_exceeded flag to overflow_seen"
        ));
        assert!(!is_context_overflow_error(
            "status messages exceed rate limits (no model context configured)"
        ));
        assert!(!is_context_overflow_error(
            "UI: error message exceeds 80 characters (display limit)"
        ));
        assert!(!is_context_overflow_error(
            "discord: message is too long (max 2000 characters)"
        ));
        assert!(!is_context_overflow_error(
            "form validation: inputs are too long (max 10 text fields)"
        ));
        assert!(!is_context_overflow_error(
            "form error: the input is too long (max 500 chars)"
        ));
        assert!(!is_context_overflow_error(
            "API: inputs exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "validation: inputs exceeded the maximum attachment size"
        ));
        assert!(!is_context_overflow_error(
            "API: input exceed rate limits for this endpoint"
        ));
        assert!(!is_context_overflow_error(
            "validation: input exceeded the maximum attachment size"
        ));
        assert!(!is_context_overflow_error(
            "docs: maximum allowed context is 128000 tokens (informational)"
        ));
        assert!(!is_context_overflow_error(
            "config: context length limit is 8192 (startup ok)"
        ));
        assert!(!is_context_overflow_error(
            "migration: dropped column old_context_budget_exceeded from events"
        ));
    }

    fn make_msg(role: &str, content: &str) -> crate::ollama::ChatMessage {
        crate::ollama::ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            images: None,
        }
    }

    #[test]
    fn truncate_tool_results_skips_system_prompt() {
        let big = "x".repeat(10_000);
        let mut msgs = vec![make_msg("system", &big), make_msg("user", "hello")];
        let n = truncate_oversized_tool_results(&mut msgs, 500);
        assert_eq!(n, 0, "system prompt at index 0 should not be truncated");
        assert_eq!(msgs[0].content.len(), 10_000);
    }

    #[test]
    fn truncate_tool_results_truncates_large_messages() {
        let big_result = "word ".repeat(2000);
        let mut msgs = vec![
            make_msg("system", "You are an AI."),
            make_msg("user", "fetch https://example.com"),
            make_msg("assistant", "FETCH_URL: https://example.com"),
            make_msg("user", &big_result),
        ];
        let n = truncate_oversized_tool_results(&mut msgs, 500);
        assert_eq!(n, 1);
        assert!(
            msgs[3].content.chars().count() < 600,
            "expected truncated msg under 600 chars, got {}",
            msgs[3].content.chars().count()
        );
        assert!(msgs[3].content.contains("[truncated from"));
    }

    #[test]
    fn truncate_tool_results_leaves_small_messages() {
        let mut msgs = vec![
            make_msg("system", "You are an AI."),
            make_msg("user", "hello"),
            make_msg("assistant", "hi there"),
        ];
        let n = truncate_oversized_tool_results(&mut msgs, 500);
        assert_eq!(n, 0);
    }

    #[test]
    fn truncate_tool_results_handles_multiple() {
        let big1 = "a".repeat(5000);
        let big2 = "b".repeat(8000);
        let mut msgs = vec![
            make_msg("system", "prompt"),
            make_msg("user", &big1),
            make_msg("assistant", "ok"),
            make_msg("user", &big2),
        ];
        let n = truncate_oversized_tool_results(&mut msgs, 1000);
        assert_eq!(n, 2);
        assert!(msgs[1].content.contains("[truncated from 5000 to 1000"));
        assert!(msgs[3].content.contains("[truncated from 8000 to 1000"));
    }

    #[test]
    fn sanitize_context_overflow_suggests_new_topic() {
        let msg = sanitize_ollama_error_for_user("Ollama error: context overflow");
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("new topic"));
        assert!(msg.contains("context window"));
    }

    #[test]
    fn sanitize_prompt_too_long_suggests_new_topic() {
        let msg = sanitize_ollama_error_for_user("Ollama error: prompt too long for context");
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("new topic"));
    }

    #[test]
    fn sanitize_maximum_context_length_tokens() {
        let msg = sanitize_ollama_error_for_user("maximum context length is 2048 tokens");
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("context window"));
    }

    #[test]
    fn sanitize_context_length_exceeded_phrase() {
        let msg = sanitize_ollama_error_for_user("context length exceeded");
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("new topic"));
    }

    #[test]
    fn sanitize_exceeds_models_context_window_phrase() {
        let msg = sanitize_ollama_error_for_user("input exceeds the model's context window");
        let text = msg.expect("overflow phrase should sanitize");
        assert!(
            text.contains("context window") && text.contains("new topic"),
            "unexpected: {text}"
        );
    }

    #[test]
    fn sanitize_requested_more_tokens_than_context_phrase() {
        let msg = sanitize_ollama_error_for_user(
            "llama runner: requested more tokens than fit in the context window",
        );
        let text = msg.expect("runner overflow phrase should sanitize");
        assert!(
            text.contains("context window") && text.contains("new topic"),
            "unexpected: {text}"
        );
    }

    #[test]
    fn sanitize_fit_in_the_context_phrase() {
        let msg =
            sanitize_ollama_error_for_user("error: prompt does not fit in the context window");
        let text = msg.expect("fit-in-context phrase should sanitize");
        assert!(
            text.contains("context window") && text.contains("new topic"),
            "unexpected: {text}"
        );
    }

    #[test]
    fn sanitize_context_size_exceeded_phrase() {
        let msg =
            sanitize_ollama_error_for_user("llama runner: context size exceeded (n_ctx=8192)");
        let text = msg.expect("context size exceeded should sanitize");
        assert!(
            text.contains("context window") && text.contains("new topic"),
            "unexpected: {text}"
        );
    }

    #[test]
    fn sanitize_role_ordering_error() {
        let msg =
            sanitize_ollama_error_for_user("Ollama error: roles must alternate user/assistant");
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("ordering"));
        assert!(msg.contains("new topic"));
    }

    #[test]
    fn sanitize_incorrect_role_error() {
        let msg = sanitize_ollama_error_for_user("incorrect role information in message");
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("ordering"));
    }

    #[test]
    fn sanitize_corrupted_session_tool_missing() {
        let msg = sanitize_ollama_error_for_user("tool call input missing from history");
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("corrupted"));
        assert!(msg.contains("new topic"));
    }

    #[test]
    fn sanitize_returns_none_for_unknown_errors() {
        assert!(sanitize_ollama_error_for_user("connection refused").is_none());
        assert!(sanitize_ollama_error_for_user("timeout").is_none());
        assert!(sanitize_ollama_error_for_user("Ollama HTTP 503: service unavailable").is_none());
        assert!(sanitize_ollama_error_for_user("Failed to send chat request").is_none());
    }
}
