//! Content reduction (truncation/summarization) and skill/JS execution helpers.
//!
//! Extracted from `ollama.rs` to keep the orchestrator focused.

use std::io::Write;
use std::process::Command;

use crate::commands::ollama_chat::{send_ollama_chat_messages, OllamaHttpQueue};

/// Heuristic: chars to tokens (conservative).
pub(crate) const CHARS_PER_TOKEN: usize = 4;

/// Same margin as [`crate::commands::context_assembler::CONTEXT_ASSEMBLER_SAFETY_TOKENS`]
/// — duplicated here to avoid a `content_reduction` ↔ `context_assembler` import cycle.
const PROACTIVE_CTX_SAFETY_TOKENS: u32 = 512;

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
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: body_truncated_for_request,
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
        },
    ];

    match send_ollama_chat_messages(
        summarization_messages,
        model_override,
        options_override,
        OllamaHttpQueue::Nested,
    )
    .await
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
/// `disclinations exceed` does not match inside `microdisclinations exceed` / `metadisclinations exceed`, and
/// `disclination exceed` does not match inside `subdisclination exceed` (left-boundary rejects
/// `predisclination exceed` and `redisclination exceed`);
/// `dislocations exceed` does not match inside `microdislocations exceed` / `metadislocations exceed`, and
/// `dislocation exceed` does not match inside `subdislocation exceed` (left-boundary rejects
/// `predislocation exceed` and `redislocation exceed`);
/// `vacancies exceed` does not match inside `microvacancies exceed` / `metavacancies exceed`, and
/// `vacancy exceed` does not match inside `subvacancy exceed` (left-boundary rejects
/// `prevacancy exceed` and `revacancy exceed`);
/// `interstitials exceed` does not match inside `microinterstitials exceed` / `metainterstitials exceed`, and
/// `interstitial exceed` does not match inside `subinterstitial exceed` (left-boundary rejects
/// `preinterstitial exceed` and `reinterstitial exceed`);
/// `voids exceed` does not match inside `microvoids exceed` / `metavoids exceed`, and
/// `void exceed` does not match inside `subvoid exceed` (left-boundary rejects `prevoid exceed` and
/// `revoid exceed`);
/// `pores exceed` does not match inside `micropores exceed` / `metapores exceed`, and
/// `pore exceed` does not match inside `subpore exceed` (left-boundary rejects `prepore exceed` and
/// `repore exceed`);
/// `inclusions exceed` does not match inside `microinclusions exceed` / `metainclusions exceed`, and
/// `inclusion exceed` does not match inside `subinclusion exceed` (left-boundary rejects
/// `preinclusion exceed` and `reinclusion exceed`);
/// `clusters exceed` does not match inside `microclusters exceed` / `metaclusters exceed`, and
/// `cluster exceed` does not match inside `subcluster exceed` (left-boundary rejects
/// `precluster exceed` and `recluster exceed`);
/// `grains exceed` does not match inside `micrograins exceed` / `metagrains exceed`, and
/// `grain exceed` does not match inside `subgrain exceed` (left-boundary rejects
/// `pregrain exceed` and `regrain exceed`);
/// `phases exceed` does not match inside `microphases exceed` / `metaphases exceed`, and
/// `phase exceed` does not match inside `subphase exceed` (left-boundary rejects
/// `prephase exceed` and `rephase exceed`);
/// `crystals exceed` does not match inside `microcrystals exceed` / `metacrystals exceed`, and
/// `crystal exceed` does not match inside `subcrystal exceed` (left-boundary rejects
/// `precrystal exceed` and `recrystal exceed`);
/// `unit cells exceed` does not match inside `microunitcells exceed` / `metaunitcells exceed`, and
/// `unit cell exceed` does not match inside `subunitcell exceed` (left-boundary rejects
/// `preunitcell exceed` and `reunitcell exceed`);
/// `primitive cells exceed` does not match inside `microprimitivecells exceed` / `metaprimitivecells exceed`, and
/// `primitive cell exceed` does not match inside `subprimitivecell exceed` (left-boundary rejects
/// `preprimitivecell exceed` and `reprimitivecell exceed`; no space between `primitive` and `cells`
/// so `microprimitivecells` does not embed the phrase `primitive cells`);
/// `supercells exceed` does not match inside `microsupercells exceed` / `metasupercells exceed`, and
/// `supercell exceed` does not match inside `subsupercell exceed` (left-boundary rejects
/// `presupercell exceed` and `resupercell exceed`; no space inside `supercells`, so `microsupercells`
/// does not embed the phrase `supercells exceed` as a spaced token sequence);
/// `k-points exceed` does not match inside `microk-points exceed` / `metak-points exceed` when the
/// prior character before `k` is alphanumeric (`microk` / `metak` runs), and
/// `k-point exceed` does not match inside `subk-point exceed` (left-boundary rejects
/// `prek-point exceed` and `rek-point exceed`; embedded `k-point exceed` inside `superk-point exceed`
/// does not match);
/// `q-points exceed` does not match inside `microq-points exceed` / `metaq-points exceed` when the
/// prior character before `q` is alphanumeric (`microq` / `metaq` runs), and
/// `q-point exceed` does not match inside `subq-point exceed` (left-boundary rejects
/// `preq-point exceed` and `req-point exceed`; embedded `q-point exceed` inside `superq-point exceed`
/// does not match);
/// `bands exceed` does not match inside `microbands exceed` / `metabands exceed`, and
/// `band exceed` does not match inside `subband exceed` (left-boundary rejects
/// `preband exceed` and `reband exceed`; embedded `band exceed` inside `superband exceed`
/// does not match);
/// `orbitals exceed` does not match inside `microorbitals exceed` / `metaorbitals exceed`, and
/// `orbital exceed` does not match inside `suborbital exceed` (left-boundary rejects
/// `preorbital exceed` and `reorbital exceed`; embedded `orbital exceed` inside `superorbital exceed`
/// does not match);
/// `basis functions exceed` does not match inside `microbasis functions exceed` / `metabasis functions exceed`, and
/// `basis function exceed` does not match inside `subbasis function exceed` (left-boundary at `basis`
/// rejects `prebasis function exceed` and `rebasis function exceed`; embedded `basis function exceed`
/// inside `superbasis function exceed` does not match; no space inside `basisfunctions`, so the
/// contiguous phrase `basis functions exceed` is absent there);
/// `auxiliary functions exceed` does not match inside `microauxiliary functions exceed` / `metaauxiliary functions exceed`, and
/// `auxiliary function exceed` does not match inside `subauxiliary function exceed` (left-boundary at `auxiliary`
/// rejects `preauxiliary function exceed` and `reauxiliary function exceed`; embedded `auxiliary function exceed`
/// inside `superauxiliary function exceed` does not match; no space inside `auxiliaryfunctions`, so the
/// contiguous phrase `auxiliary functions exceed` is absent there);
/// `primitive gaussians exceed` does not match inside `microprimitive gaussians exceed` / `metaprimitive gaussians exceed`, and
/// `primitive gaussian exceed` does not match inside `subprimitive gaussian exceed` (left-boundary at `primitive`
/// rejects `preprimitive gaussian exceed` and `reprimitive gaussian exceed`; embedded `primitive gaussian exceed`
/// inside `superprimitive gaussian exceed` does not match; no space inside `primitivegaussians`, so the
/// contiguous phrase `primitive gaussians exceed` is absent there);
/// `contracted gaussians exceed` does not match inside `microcontracted gaussians exceed` / `metacontracted gaussians exceed`, and
/// `contracted gaussian exceed` does not match inside `subcontracted gaussian exceed` (left-boundary at `contracted`
/// rejects `precontracted gaussian exceed` and `recontracted gaussian exceed`; embedded `contracted gaussian exceed`
/// inside `supercontracted gaussian exceed` does not match; no space inside `contractedgaussians`, so the
/// contiguous phrase `contracted gaussians exceed` is absent there);
/// `spherical gaussians exceed` does not match inside `microspherical gaussians exceed` / `metaspherical gaussians exceed`, and
/// `spherical gaussian exceed` does not match inside `subspherical gaussian exceed` (left-boundary at `spherical`
/// rejects `prespherical gaussian exceed` and `respherical gaussian exceed`; embedded `spherical gaussian exceed`
/// inside `superspherical gaussian exceed` does not match; no space inside `sphericalgaussians`, so the
/// contiguous phrase `spherical gaussians exceed` is absent there);
/// `cartesian gaussians exceed` does not match inside `microcartesian gaussians exceed` / `metacartesian gaussians exceed`, and
/// `cartesian gaussian exceed` does not match inside `subcartesian gaussian exceed` (left-boundary at `cartesian`
/// rejects `precartesian gaussian exceed` and `recartesian gaussian exceed`; embedded `cartesian gaussian exceed`
/// inside `supercartesian gaussian exceed` does not match; no space inside `cartesiangaussians`, so the
/// contiguous phrase `cartesian gaussians exceed` is absent there);
/// `gaussian shells exceed` does not match inside `microgaussian shells exceed` / `metagaussian shells exceed`, and
/// `gaussian shell exceed` does not match inside `subgaussian shell exceed` (left-boundary at `gaussian`
/// rejects `pregaussian shell exceed` and `regaussian shell exceed`; embedded `gaussian shell exceed`
/// inside `supergaussian shell exceed` does not match; no space inside `gaussianshells`, so the
/// contiguous phrase `gaussian shells exceed` is absent there);
/// `density matrices exceed` does not match inside `microdensity matrices exceed` / `metadensity matrices exceed`, and
/// `density matrix exceed` does not match inside `subdensity matrix exceed` (left-boundary at `density`
/// rejects `predensity matrix exceed` and `redensity matrix exceed`; embedded `density matrix exceed`
/// inside `superdensity matrix exceed` does not match; no space inside `densitymatrices`, so the
/// contiguous phrase `density matrices exceed` is absent there);
/// `molecular orbitals exceed` does not match inside `micromolecularorbitals exceed` / `metamolecularorbitals exceed`, and
/// `molecular orbital exceed` does not match inside `submolecular orbital exceed` (left-boundary at `molecular`
/// rejects `premolecular orbital exceed` and `remolecular orbital exceed`; embedded `molecular orbital exceed`
/// inside `supermolecularorbital exceed` does not match; no space inside `molecularorbitals`, so the
/// contiguous phrase `molecular orbitals exceed` is absent there; a spaced `micromolecular orbitals exceed`
/// is not listed here because the generic `orbitals exceed` arm can match at the boundary before `orbitals`;
/// similarly `supermolecular orbital exceed` / `premolecular orbital exceed` are not listed because the generic
/// `orbital exceed` arm matches at the boundary before `orbital` when a context slot is present);
/// `atomic orbitals exceed` does not match inside `microatomicorbitals exceed` / `metaatomicorbitals exceed`, and
/// `atomic orbital exceed` does not match inside `subatomic orbital exceed` (left-boundary at `atomic`
/// rejects `preatomic orbital exceed` and `reatomic orbital exceed`; embedded `atomic orbital exceed`
/// inside `superatomicorbital exceed` does not match; no space inside `atomicorbitals`, so the
/// contiguous phrase `atomic orbitals exceed` is absent there; a spaced `micro atomic orbitals exceed`
/// is not listed here because the generic `orbitals exceed` arm can match at the boundary before `orbitals`;
/// `subatomic …` is rejected at the boundary before `atomic` (the `b` in `sub` is an ident continuation).
/// `wave functions exceed` does not match inside `microwave functions exceed` / `shortwave functions exceed`, and
/// `wave function exceed` does not match inside `subwave function exceed` (left-boundary at `wave`
/// rejects `prewave function exceed` and `rewave function exceed`; there is no bare `functions exceed` /
/// `function exceed` arm—only qualified phrases such as `basis functions exceed`—so `superwavefunction exceed`
/// does not match via a generic `function` tail; no space inside `wavefunctions`, so the contiguous phrase
/// `wave functions exceed` is absent there; a spaced `micro wave functions …` still matches at the boundary
/// before `wave`);
/// `slater determinants exceed` does not match inside `microslater determinants exceed` / `metaslater determinants exceed`, and
/// `slater determinant exceed` does not match inside `subslater determinant exceed` (left-boundary at `slater`
/// rejects `preslater determinant exceed` and `reslater determinant exceed`; embedded `slater determinant exceed`
/// inside `superslater determinant exceed` does not match (the `r` in `super` is an ident continuation before `slater`);
/// no space inside `slaterdeterminants`, so the contiguous phrase `slater determinants exceed` is absent in
/// `microslaterdeterminants exceed` / `metaslaterdeterminants exceed`;
/// `superslaterdeterminant exceed` has no contiguous `slater determinant exceed` substring (no space between `slater` and `determinant`));
/// `configuration state functions exceed` does not match inside `microconfiguration state functions exceed` / `metaconfiguration state functions exceed`, and
/// `configuration state function exceed` does not match inside `subconfiguration state function exceed` (left-boundary at `configuration`
/// rejects `preconfiguration state function exceed` and `reconfiguration state function exceed` when `configuration` is embedded in a longer ident;
/// embedded `configuration state function exceed` inside `superconfiguration state function exceed` does not match (the `r` in `super` is an ident
/// continuation before `configuration`); no space inside `configurationstatefunctions`, so the contiguous phrase `configuration state functions exceed` is absent in
/// `microconfigurationstatefunctions exceed` / `metaconfigurationstatefunctions exceed`;
/// `superconfigurationstatefunction exceed` has no contiguous `configuration state function exceed` substring (no space between `configuration` and `state`));
/// `csf coefficients exceed` does not match inside `microcsf coefficients exceed` / `metacsf coefficients exceed`, and
/// `csf coefficient exceed` does not match inside `subcsf coefficient exceed` (left-boundary at `csf`
/// rejects `precsf coefficient exceed` and `recsf coefficient exceed` when `csf` is embedded in a longer ident;
/// embedded `csf coefficient exceed` inside `supercsf coefficient exceed` does not match (the `r` in `super` is an ident
/// continuation before `csf`); no space inside `csfcoefficients`, so the contiguous phrase `csf coefficients exceed` is absent in
/// `microcsfcoefficients exceed` / `metacsfcoefficients exceed`;
/// `supercsfcoefficient exceed` has no contiguous `csf coefficient exceed` substring (no space between `csf` and `coefficient`));
/// `ci coefficients exceed` does not match inside `microci coefficients exceed` / `metaci coefficients exceed`, and
/// `ci coefficient exceed` does not match inside `subci coefficient exceed` (left-boundary at `ci`
/// rejects `preci coefficient exceed` and `reci coefficient exceed` when `ci` is embedded in a longer ident;
/// embedded `ci coefficient exceed` inside `superci coefficient exceed` does not match (the `r` in `super` is an ident
/// continuation before `ci`); no space inside `cicoefficients`, so the contiguous phrase `ci coefficients exceed` is absent in
/// `microcicoefficients exceed` / `metacicoefficients exceed`;
/// `supercicoefficient exceed` has no contiguous `ci coefficient exceed` substring (no space between `ci` and `coefficient`));
/// `mo coefficients exceed` does not match inside `micromocoefficients exceed` / `metamocoefficients exceed`, and
/// `mo coefficient exceed` does not match inside `submo coefficient exceed` (left-boundary at `mo`
/// rejects `premo coefficient exceed` and `remo coefficient exceed` when `mo` is embedded in a longer ident;
/// embedded `mo coefficient exceed` inside `supermo coefficient exceed` does not match (the `r` in `super` is an ident
/// continuation before `mo`); no space inside `mocoefficients`, so the contiguous phrase `mo coefficients exceed` is absent in
/// `micromocoefficients exceed` / `metamocoefficients exceed`;
/// `supermocoefficient exceed` has no contiguous `mo coefficient exceed` substring (no space between `mo` and `coefficient`));
/// `natural orbitals exceed` does not match inside `micronaturalorbitals exceed` / `metanaturalorbitals exceed`, and
/// `natural orbital exceed` does not match inside `subnatural orbital exceed` (left-boundary at `natural`
/// rejects `prenaturalorbital exceed` and `renaturalorbital exceed`; embedded `natural orbital exceed`
/// inside `supernaturalorbital exceed` does not match; no space inside `naturalorbitals`, so the
/// contiguous phrase `natural orbitals exceed` is absent there; a spaced `micronatural orbitals exceed`
/// is not listed here because the generic `orbitals exceed` arm can match at the boundary before `orbitals`;
/// similarly `supernatural orbital exceed` / `prenatural orbital exceed` are not listed because the generic
/// `orbital exceed` arm matches at the boundary before `orbital` when a context slot is present);
/// `occupied orbitals exceed` does not match inside `microoccupiedorbitals exceed` / `metaoccupiedorbitals exceed`, and
/// `occupied orbital exceed` does not match inside `suboccupied orbital exceed` (left-boundary at `occupied`
/// rejects embedded `occupied` after an ident char (`preoccupied …`, `unoccupied …` on the qualified `occupied orbitals` arm);
/// a spaced `preoccupied orbitals …` / `unoccupied orbitals …` with a context slot can still match the generic
/// `orbitals exceed` arm at the boundary before `orbitals` (parallel to `natural orbitals exceed`);
/// no space inside `occupiedorbitals`, so the contiguous phrase `occupied orbitals exceed` is absent there;
/// a spaced `micro occupied orbitals …` still matches at the boundary before `occupied`);
/// `canonical orbitals exceed` does not match inside `microcanonicalorbitals exceed` / `metacanonicalorbitals exceed`, and
/// `canonical orbital exceed` does not match inside `subcanonical orbital exceed` (left-boundary at `canonical`
/// rejects `precanonicalorbital exceed` and `recanonicalorbital exceed`; embedded `canonical orbital exceed`
/// inside `supercanonicalorbital exceed` does not match; no space inside `canonicalorbitals`, so the
/// contiguous phrase `canonical orbitals exceed` is absent there; a spaced `micro canonical orbitals …`
/// still matches at the boundary before `canonical`; `microcanonical` as one token breaks the boundary before `canonical`);
/// `electrons exceed` does not match inside `microelectrons exceed` / `metaelectrons exceed`, and
/// `electron exceed` does not match inside `subelectron exceed` (left-boundary rejects
/// `preelectron exceed` and `reelectron exceed`; embedded `electron exceed` inside `superelectron exceed`
/// does not match);
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
        if i > 0
            && haystack[..i]
                .chars()
                .next_back()
                .is_some_and(ident_continue)
        {
            continue;
        }
        if preceding_ascii_ident_token(haystack, i)
            .is_some_and(|t| t != "total" && t.ends_with("total"))
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
        if i > 0
            && haystack[..i]
                .chars()
                .next_back()
                .is_some_and(ident_continue)
        {
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
        // Plural / singular "disclinations / disclination exceed(s/ed)" (FEAT-D429). Parallel to
        // `vortices exceed` / `vortex exceed`. `disclination exceed` matches present/past via `exceed`
        // prefix of `exceeds` / `exceeded` and does not substring-match plural `disclinations exceed`
        // (the `s` after `disclination` breaks `disclination` + space + `exceed`). Ident-boundary so
        // `microdisclinations exceed` / `metadisclinations exceed` / `subdisclination exceed` do not
        // false-positive; `predisclination exceed` and `redisclination exceed` are rejected the same way.
        // Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `disclinations exceed`
        // rate limits, per-domain / line-defect caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "disclinations exceed")
            || contains_phrase_after_ident_boundary(&lower, "disclinations exceeded")
            || contains_phrase_after_ident_boundary(&lower, "disclination exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "dislocations / dislocation exceed(s/ed)" (FEAT-D430). Parallel to
        // `disclinations exceed` / `disclination exceed`. `dislocation exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `dislocations exceed` (the `s` after `dislocation` breaks `dislocation` + space + `exceed`).
        // Ident-boundary so `microdislocations exceed` / `metadislocations exceed` / `subdislocation exceed`
        // do not false-positive; `predislocation exceed` and `redislocation exceed` are rejected the same way;
        // embedded `dislocation exceed` inside `superdislocation exceed` does not match. Same explicit
        // context-slot phrases as `messages exceed`. Negatives: HTTP `dislocations exceed` rate limits,
        // per-slip-system / Burgers-vector caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "dislocations exceed")
            || contains_phrase_after_ident_boundary(&lower, "dislocations exceeded")
            || contains_phrase_after_ident_boundary(&lower, "dislocation exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "vacancies / vacancy exceed(s/ed)" (FEAT-D431). Parallel to
        // `dislocations exceed` / `dislocation exceed`. `vacancy exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `vacancies exceed` (the `s` after `vacancy` breaks `vacancy` + space + `exceed`).
        // Ident-boundary so `microvacancies exceed` / `metavacancies exceed` / `subvacancy exceed`
        // do not false-positive; `prevacancy exceed` and `revacancy exceed` are rejected the same way;
        // embedded `vacancy exceed` inside `supervacancy exceed` does not match. Same explicit
        // context-slot phrases as `messages exceed`. Negatives: HTTP `vacancies exceed` rate limits,
        // per-lattice / formation-energy caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "vacancies exceed")
            || contains_phrase_after_ident_boundary(&lower, "vacancies exceeded")
            || contains_phrase_after_ident_boundary(&lower, "vacancy exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "interstitials / interstitial exceed(s/ed)" (FEAT-D432). Parallel to
        // `vacancies exceed` / `vacancy exceed`. `interstitial exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `interstitials exceed` (the `s` after `interstitial` breaks `interstitial` + space + `exceed`).
        // Ident-boundary so `microinterstitials exceed` / `metainterstitials exceed` / `subinterstitial exceed`
        // do not false-positive; `preinterstitial exceed` and `reinterstitial exceed` are rejected the same way;
        // embedded `interstitial exceed` inside `superinterstitial exceed` does not match. Same explicit
        // context-slot phrases as `messages exceed`. Negatives: HTTP `interstitials exceed` rate limits,
        // per-lattice / migration-barrier caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "interstitials exceed")
            || contains_phrase_after_ident_boundary(&lower, "interstitials exceeded")
            || contains_phrase_after_ident_boundary(&lower, "interstitial exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "voids / void exceed(s/ed)" (FEAT-D433). Parallel to
        // `interstitials exceed` / `interstitial exceed`. `void exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `voids exceed` (the `s` after `void` breaks `void` + space + `exceed`).
        // Ident-boundary so `microvoids exceed` / `metavoids exceed` / `subvoid exceed` do not
        // false-positive; `prevoid exceed` and `revoid exceed` are rejected the same way; embedded
        // `void exceed` inside `supervoid exceed` does not match. Same explicit context-slot phrases
        // as `messages exceed`. Negatives: HTTP `voids exceed` rate limits, per-cell / free-volume
        // caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "voids exceed")
            || contains_phrase_after_ident_boundary(&lower, "voids exceeded")
            || contains_phrase_after_ident_boundary(&lower, "void exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "pores / pore exceed(s/ed)" (FEAT-D434). Parallel to
        // `voids exceed` / `void exceed`. `pore exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `pores exceed` (the `s` after `pore` breaks `pore` + space + `exceed`).
        // Ident-boundary so `micropores exceed` / `metapores exceed` / `subpore exceed` do not
        // false-positive; `prepore exceed` and `repore exceed` are rejected the same way; embedded
        // `pore exceed` inside `superpore exceed` does not match. Same explicit context-slot phrases
        // as `messages exceed`. Negatives: HTTP `pores exceed` rate limits, per-sample / porosity
        // caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "pores exceed")
            || contains_phrase_after_ident_boundary(&lower, "pores exceeded")
            || contains_phrase_after_ident_boundary(&lower, "pore exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "inclusions / inclusion exceed(s/ed)" (FEAT-D435). Parallel to
        // `pores exceed` / `pore exceed`. `inclusion exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `inclusions exceed` (the `s` after `inclusion` breaks `inclusion` + space + `exceed`).
        // Ident-boundary so `microinclusions exceed` / `metainclusions exceed` / `subinclusion exceed` do not
        // false-positive; `preinclusion exceed` and `reinclusion exceed` are rejected the same way; embedded
        // `inclusion exceed` inside `superinclusion exceed` does not match. Same explicit context-slot phrases
        // as `messages exceed`. Negatives: HTTP `inclusions exceed` rate limits, per-volume / second-phase
        // caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "inclusions exceed")
            || contains_phrase_after_ident_boundary(&lower, "inclusions exceeded")
            || contains_phrase_after_ident_boundary(&lower, "inclusion exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "clusters / cluster exceed(s/ed)" (FEAT-D436). Parallel to
        // `inclusions exceed` / `inclusion exceed`. `cluster exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `clusters exceed` (the `s` after `cluster` breaks `cluster` + space + `exceed`).
        // Ident-boundary so `microclusters exceed` / `metaclusters exceed` / `subcluster exceed` do not
        // false-positive; `precluster exceed` and `recluster exceed` are rejected the same way; embedded
        // `cluster exceed` inside `supercluster exceed` does not match. Same explicit context-slot phrases
        // as `messages exceed`. Negatives: HTTP `clusters exceed` rate limits, per-graph / linkage
        // caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "clusters exceed")
            || contains_phrase_after_ident_boundary(&lower, "clusters exceeded")
            || contains_phrase_after_ident_boundary(&lower, "cluster exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "grains / grain exceed(s/ed)" (FEAT-D437). Parallel to
        // `clusters exceed` / `cluster exceed`. `grain exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `grains exceed` (the `s` after `grain` breaks `grain` + space + `exceed`).
        // Ident-boundary so `micrograins exceed` / `metagrains exceed` / `subgrain exceed` do not
        // false-positive; `pregrain exceed` and `regrain exceed` are rejected the same way; embedded
        // `grain exceed` inside `supergrain exceed` does not match. Same explicit context-slot phrases
        // as `messages exceed`. Negatives: HTTP `grains exceed` rate limits, per-polycrystal /
        // grain-boundary caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "grains exceed")
            || contains_phrase_after_ident_boundary(&lower, "grains exceeded")
            || contains_phrase_after_ident_boundary(&lower, "grain exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "phases / phase exceed(s/ed)" (FEAT-D438). Parallel to
        // `grains exceed` / `grain exceed`. `phase exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `phases exceed` (the `s` after `phase` breaks `phase` + space + `exceed`).
        // Ident-boundary so `microphases exceed` / `metaphases exceed` / `subphase exceed` do not
        // false-positive; `prephase exceed` and `rephase exceed` are rejected the same way; embedded
        // `phase exceed` inside `superphase exceed` does not match. Same explicit context-slot phrases
        // as `messages exceed`. Negatives: HTTP `phases exceed` rate limits, per-sample / phase-fraction
        // or coexistence caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "phases exceed")
            || contains_phrase_after_ident_boundary(&lower, "phases exceeded")
            || contains_phrase_after_ident_boundary(&lower, "phase exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "crystals / crystal exceed(s/ed)" (FEAT-D439). Parallel to
        // `phases exceed` / `phase exceed`. `crystal exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `crystals exceed` (the `s` after `crystal` breaks `crystal` + space + `exceed`).
        // Ident-boundary so `microcrystals exceed` / `metacrystals exceed` / `subcrystal exceed` do not
        // false-positive; `precrystal exceed` and `recrystal exceed` are rejected the same way; embedded
        // `crystal exceed` inside `supercrystal exceed` does not match. Same explicit context-slot phrases
        // as `messages exceed`. Negatives: HTTP `crystals exceed` rate limits, per-unit-cell / lattice
        // caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "crystals exceed")
            || contains_phrase_after_ident_boundary(&lower, "crystals exceeded")
            || contains_phrase_after_ident_boundary(&lower, "crystal exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "unit cells / unit cell exceed(s/ed)" (FEAT-D440). Parallel to
        // `crystals exceed` / `crystal exceed`. `unit cell exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `unit cells exceed` (the `s` in `cells` prevents the singular `cell` + space + `exceed`
        // path from aligning inside the plural phrase).
        // Ident-boundary at `unit` so `microunitcells exceed` / `metaunitcells exceed` /
        // `subunitcell exceed` do not false-positive (`microunit cells …` is not listed: a space
        // before `cells` would match the separate `cells exceed` arm). `preunitcell exceed` and
        // `reunitcell exceed` are rejected the same way; embedded `unit cell exceed` inside
        // `superunitcell exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `unit cells exceed` rate limits, per-lattice /
        // basis-vector caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "unit cells exceed")
            || contains_phrase_after_ident_boundary(&lower, "unit cells exceeded")
            || contains_phrase_after_ident_boundary(&lower, "unit cell exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "primitive cells / primitive cell exceed(s/ed)" (FEAT-D441). Parallel to
        // `unit cells exceed` / `unit cell exceed`. `primitive cell exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `primitive cells exceed` (the `s` in `cells` prevents the singular `cell` + space + `exceed`
        // path from aligning inside the plural phrase).
        // Ident-boundary at `primitive` so `microprimitivecells exceed` / `metaprimitivecells exceed` /
        // `subprimitivecell exceed` do not false-positive (no space inside `primitivecells`, so the
        // phrase `primitive cells exceed` is absent; a spaced form `micro primitive cells …` still
        // matches at the word boundary before `primitive`). `preprimitivecell exceed` and
        // `reprimitivecell exceed` are rejected the same way; embedded `primitive cell exceed` inside
        // `superprimitivecell exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `primitive cells exceed` rate limits, per-basis /
        // Wigner–Seitz caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "primitive cells exceed")
            || contains_phrase_after_ident_boundary(&lower, "primitive cells exceeded")
            || contains_phrase_after_ident_boundary(&lower, "primitive cell exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "supercells / supercell exceed(s/ed)" (FEAT-D442). Parallel to
        // `primitive cells exceed` / `primitive cell exceed`. `supercell exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `supercells exceed` (the `s` in `cells` prevents the singular `cell` + space + `exceed`
        // path from aligning inside the plural phrase).
        // Ident-boundary at the start of `supercells` / `supercell` so `microsupercells exceed` /
        // `metasupercells exceed` / `subsupercell exceed` do not false-positive (no space inside
        // `supercells`, so the contiguous phrase `supercells exceed` is absent there; a spaced form
        // `micro supercells …` still matches at the boundary before `supercells`). `presupercell exceed` and
        // `resupercell exceed` are rejected the same way; embedded `supercell exceed` inside
        // `supersupercell exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `supercells exceed` rate limits, per-k-point /
        // replica-exchange caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "supercells exceed")
            || contains_phrase_after_ident_boundary(&lower, "supercells exceeded")
            || contains_phrase_after_ident_boundary(&lower, "supercell exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "k-points / k-point exceed(s/ed)" (FEAT-D443). Parallel to
        // `supercells exceed` / `supercell exceed`. `k-point exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `k-points exceed` (the `s` after `k-point` in `k-points` breaks the singular path).
        // Ident-boundary before `k` so `microk-points exceed` / `metak-points exceed` /
        // `subk-point exceed` do not false-positive (hyphenated `micro k-points …` still matches).
        // `prek-point exceed` and `rek-point exceed` are rejected the same way; embedded
        // `k-point exceed` inside `superk-point exceed` does not match. Same explicit context-slot
        // phrases as `messages exceed`. Negatives: HTTP `k-points exceed` rate limits, per-mesh /
        // Monkhorst–Pack caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "k-points exceed")
            || contains_phrase_after_ident_boundary(&lower, "k-points exceeded")
            || contains_phrase_after_ident_boundary(&lower, "k-point exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "q-points / q-point exceed(s/ed)" (FEAT-D444). Parallel to
        // `k-points exceed` / `k-point exceed`. `q-point exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `q-points exceed` (the `s` after `q-point` in `q-points` breaks the singular path).
        // Ident-boundary before `q` so `microq-points exceed` / `metaq-points exceed` /
        // `subq-point exceed` do not false-positive (hyphenated `micro q-points …` still matches).
        // `preq-point exceed` and `req-point exceed` are rejected the same way; embedded
        // `q-point exceed` inside `superq-point exceed` does not match. Same explicit context-slot
        // phrases as `messages exceed`. Negatives: HTTP `q-points exceed` rate limits, per-mesh /
        // phonon-branch caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "q-points exceed")
            || contains_phrase_after_ident_boundary(&lower, "q-points exceeded")
            || contains_phrase_after_ident_boundary(&lower, "q-point exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "bands / band exceed(s/ed)" (FEAT-D445). Parallel to
        // `q-points exceed` / `q-point exceed`. `band exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `bands exceed` (the `s` after `band` breaks `band` + space + `exceed`).
        // Ident-boundary at `bands` / `band` so `microbands exceed` / `metabands exceed` /
        // `subband exceed` do not false-positive; `preband exceed` and `reband exceed` are rejected
        // the same way; embedded `band exceed` inside `superband exceed` does not match. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `bands exceed` rate
        // limits, per-k-path / empty-state caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "bands exceed")
            || contains_phrase_after_ident_boundary(&lower, "bands exceeded")
            || contains_phrase_after_ident_boundary(&lower, "band exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "orbitals / orbital exceed(s/ed)" (FEAT-D446). Parallel to
        // `bands exceed` / `band exceed`. `orbital exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `orbitals exceed` (the `s` after `orbital` breaks `orbital` + space + `exceed`).
        // Ident-boundary at `orbitals` / `orbital` so `microorbitals exceed` / `metaorbitals exceed` /
        // `suborbital exceed` do not false-positive; `preorbital exceed` and `reorbital exceed` are rejected
        // the same way; embedded `orbital exceed` inside `superorbital exceed` does not match. Same
        // explicit context-slot phrases as `messages exceed`. Negatives: HTTP `orbitals exceed` rate
        // limits, per-basis-set / active-space caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "orbitals exceed")
            || contains_phrase_after_ident_boundary(&lower, "orbitals exceeded")
            || contains_phrase_after_ident_boundary(&lower, "orbital exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "basis functions / basis function exceed(s/ed)" (FEAT-D447). Parallel to
        // `orbitals exceed` / `orbital exceed`. `basis function exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `basis functions exceed` (the `s` in `functions` prevents the singular `function` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `basis` so `microbasis functions exceed` / `metabasis functions exceed` /
        // `subbasis function exceed` do not false-positive (no space inside `basisfunctions`, so the
        // phrase `basis functions exceed` is absent there; a spaced form `micro basis functions …`
        // still matches at the boundary before `basis`). `prebasis function exceed` and
        // `rebasis function exceed` are rejected the same way; embedded `basis function exceed` inside
        // `superbasis function exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `basis functions exceed` rate limits, per-zeta /
        // auxiliary-basis caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "basis functions exceed")
            || contains_phrase_after_ident_boundary(&lower, "basis functions exceeded")
            || contains_phrase_after_ident_boundary(&lower, "basis function exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "auxiliary functions / auxiliary function exceed(s/ed)" (FEAT-D449). Parallel to
        // `basis functions exceed` / `basis function exceed`. `auxiliary function exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `auxiliary functions exceed` (the `s` in `functions` prevents the singular `function` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `auxiliary` so `microauxiliary functions exceed` / `metaauxiliary functions exceed` /
        // `subauxiliary function exceed` do not false-positive (no space inside `auxiliaryfunctions`, so the
        // phrase `auxiliary functions exceed` is absent there; a spaced form `micro auxiliary functions …`
        // still matches at the boundary before `auxiliary`). `preauxiliary function exceed` and
        // `reauxiliary function exceed` are rejected the same way; embedded `auxiliary function exceed` inside
        // `superauxiliary function exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `auxiliary functions exceed` rate limits, per-RI / JK-density caps,
        // etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "auxiliary functions exceed")
            || contains_phrase_after_ident_boundary(&lower, "auxiliary functions exceeded")
            || contains_phrase_after_ident_boundary(&lower, "auxiliary function exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "primitive gaussians / primitive gaussian exceed(s/ed)" (FEAT-D450). Parallel to
        // `auxiliary functions exceed` / `auxiliary function exceed`. `primitive gaussian exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `primitive gaussians exceed` (the `s` in `gaussians` prevents the singular `gaussian` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `primitive` so `microprimitive gaussians exceed` / `metaprimitive gaussians exceed` /
        // `subprimitive gaussian exceed` do not false-positive (no space inside `primitivegaussians`, so the
        // phrase `primitive gaussians exceed` is absent there; a spaced form `micro primitive gaussians …`
        // still matches at the boundary before `primitive`). `preprimitive gaussian exceed` and
        // `reprimitive gaussian exceed` are rejected the same way; embedded `primitive gaussian exceed` inside
        // `superprimitive gaussian exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `primitive gaussians exceed` rate limits, per-shell / PGF-basis caps,
        // etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "primitive gaussians exceed")
            || contains_phrase_after_ident_boundary(&lower, "primitive gaussians exceeded")
            || contains_phrase_after_ident_boundary(&lower, "primitive gaussian exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "contracted gaussians / contracted gaussian exceed(s/ed)" (FEAT-D451). Parallel to
        // `primitive gaussians exceed` / `primitive gaussian exceed`. `contracted gaussian exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `contracted gaussians exceed` (the `s` in `gaussians` prevents the singular `gaussian` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `contracted` so `microcontracted gaussians exceed` / `metacontracted gaussians exceed` /
        // `subcontracted gaussian exceed` do not false-positive (no space inside `contractedgaussians`, so the
        // phrase `contracted gaussians exceed` is absent there; a spaced form `micro contracted gaussians …`
        // still matches at the boundary before `contracted`). `precontracted gaussian exceed` and
        // `recontracted gaussian exceed` are rejected the same way; embedded `contracted gaussian exceed` inside
        // `supercontracted gaussian exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `contracted gaussians exceed` rate limits, per-shell / CGF-basis caps,
        // etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "contracted gaussians exceed")
            || contains_phrase_after_ident_boundary(&lower, "contracted gaussians exceeded")
            || contains_phrase_after_ident_boundary(&lower, "contracted gaussian exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "spherical gaussians / spherical gaussian exceed(s/ed)" (FEAT-D452). Parallel to
        // `contracted gaussians exceed` / `contracted gaussian exceed`. `spherical gaussian exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `spherical gaussians exceed` (the `s` in `gaussians` prevents the singular `gaussian` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `spherical` so `microspherical gaussians exceed` / `metaspherical gaussians exceed` /
        // `subspherical gaussian exceed` do not false-positive (no space inside `sphericalgaussians`, so the
        // phrase `spherical gaussians exceed` is absent there; a spaced form `micro spherical gaussians …`
        // still matches at the boundary before `spherical`). `prespherical gaussian exceed` and
        // `respherical gaussian exceed` are rejected the same way; embedded `spherical gaussian exceed` inside
        // `superspherical gaussian exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `spherical gaussians exceed` rate limits, per-shell / SGF-basis caps,
        // etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "spherical gaussians exceed")
            || contains_phrase_after_ident_boundary(&lower, "spherical gaussians exceeded")
            || contains_phrase_after_ident_boundary(&lower, "spherical gaussian exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "cartesian gaussians / cartesian gaussian exceed(s/ed)" (FEAT-D453). Parallel to
        // `spherical gaussians exceed` / `spherical gaussian exceed`. `cartesian gaussian exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `cartesian gaussians exceed` (the `s` in `gaussians` prevents the singular `gaussian` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `cartesian` so `microcartesian gaussians exceed` / `metacartesian gaussians exceed` /
        // `subcartesian gaussian exceed` do not false-positive (no space inside `cartesiangaussians`, so the
        // phrase `cartesian gaussians exceed` is absent there; a spaced form `micro cartesian gaussians …`
        // still matches at the boundary before `cartesian`). `precartesian gaussian exceed` and
        // `recartesian gaussian exceed` are rejected the same way; embedded `cartesian gaussian exceed` inside
        // `supercartesian gaussian exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `cartesian gaussians exceed` rate limits, per-center / Cartesian-GTO caps,
        // etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "cartesian gaussians exceed")
            || contains_phrase_after_ident_boundary(&lower, "cartesian gaussians exceeded")
            || contains_phrase_after_ident_boundary(&lower, "cartesian gaussian exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "gaussian shells / gaussian shell exceed(s/ed)" (FEAT-D454). Parallel to
        // `cartesian gaussians exceed` / `cartesian gaussian exceed`. `gaussian shell exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `gaussian shells exceed` (the `s` in `shells` prevents the singular `shell` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `gaussian` so `microgaussian shells exceed` / `metagaussian shells exceed` /
        // `subgaussian shell exceed` do not false-positive (no space inside `gaussianshells`, so the
        // phrase `gaussian shells exceed` is absent there; a spaced form `micro gaussian shells …`
        // still matches at the boundary before `gaussian`). `pregaussian shell exceed` and
        // `regaussian shell exceed` are rejected the same way; embedded `gaussian shell exceed` inside
        // `supergaussian shell exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `gaussian shells exceed` rate limits, per-angular-momentum /
        // shell-block caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "gaussian shells exceed")
            || contains_phrase_after_ident_boundary(&lower, "gaussian shells exceeded")
            || contains_phrase_after_ident_boundary(&lower, "gaussian shell exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "density matrices / density matrix exceed(s/ed)" (FEAT-D455). Parallel to
        // `gaussian shells exceed` / `gaussian shell exceed`. `density matrix exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `density matrices exceed` (the `s` in `matrices` prevents the singular `matrix` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `density` so `microdensity matrices exceed` / `metadensity matrices exceed` /
        // `subdensity matrix exceed` do not false-positive (no space inside `densitymatrices`, so the
        // phrase `density matrices exceed` is absent there; a spaced form `micro density matrices …`
        // still matches at the boundary before `density`). `predensity matrix exceed` and
        // `redensity matrix exceed` are rejected the same way; embedded `density matrix exceed` inside
        // `superdensity matrix exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `density matrices exceed` rate limits, per-orbital /
        // N-representability caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "density matrices exceed")
            || contains_phrase_after_ident_boundary(&lower, "density matrices exceeded")
            || contains_phrase_after_ident_boundary(&lower, "density matrix exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "molecular orbitals / molecular orbital exceed(s/ed)" (FEAT-D456). Parallel to
        // `density matrices exceed` / `density matrix exceed`. `molecular orbital exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `molecular orbitals exceed` (the `s` in `orbitals` prevents the singular `orbital` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `molecular` so `submolecular orbital exceed` does not false-positive (no space inside
        // `molecularorbitals`, so the phrase `molecular orbitals exceed` is absent in `micromolecularorbitals exceed` /
        // `metamolecularorbitals exceed`, which also avoids the generic `orbitals exceed` arm; a spaced
        // `micro molecular orbitals …` still matches at the boundary before `molecular`). `premolecular orbital exceed` and
        // `remolecular orbital exceed` are rejected the same way; embedded `molecular orbital exceed` inside
        // `supermolecularorbital exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `molecular orbitals exceed` rate limits, active-space /
        // MO-basis caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "molecular orbitals exceed")
            || contains_phrase_after_ident_boundary(&lower, "molecular orbitals exceeded")
            || contains_phrase_after_ident_boundary(&lower, "molecular orbital exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "atomic orbitals / atomic orbital exceed(s/ed)" (FEAT-D457). Parallel to
        // `molecular orbitals exceed` / `molecular orbital exceed`. `atomic orbital exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `atomic orbitals exceed` (the `s` in `orbitals` prevents the singular `orbital` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `atomic` so `subatomic orbital exceed` does not false-positive (no space inside
        // `atomicorbitals`, so the phrase `atomic orbitals exceed` is absent in `microatomicorbitals exceed` /
        // `metaatomicorbitals exceed`, which also avoids the generic `orbitals exceed` arm; a spaced
        // `micro atomic orbitals …` still matches at the boundary before `atomic`). `preatomic orbital exceed` and
        // `reatomic orbital exceed` are rejected the same way; embedded `atomic orbital exceed` inside
        // `superatomicorbital exceed` does not match. Same explicit context-slot phrases as
        // `messages exceed`. Negatives: HTTP `atomic orbitals exceed` rate limits, AO-basis /
        // valence-shell caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "atomic orbitals exceed")
            || contains_phrase_after_ident_boundary(&lower, "atomic orbitals exceeded")
            || contains_phrase_after_ident_boundary(&lower, "atomic orbital exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "wave functions / wave function exceed(s/ed)" (FEAT-D458). Parallel to
        // `atomic orbitals exceed` / `atomic orbital exceed`. `wave function exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `wave functions exceed` (the `s` in `functions` prevents the singular `function` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `wave` so `microwave functions exceed` / `shortwave functions exceed` /
        // `subwave function exceed` do not false-positive (no space inside `wavefunctions`, so the
        // phrase `wave functions exceed` is absent in `microwavefunctions exceed` / `metawavefunctions exceed`,
        // which also avoids qualified `… functions exceed` arms such as `basis functions exceed`; a spaced
        // `micro wave functions …` still matches at the boundary before `wave`). `prewave function exceed` and
        // `rewave function exceed` are rejected the same way; `superwavefunction exceed` has no contiguous
        // `wave function exceed` substring (no space between `wave` and `function`).
        // Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `wave functions exceed`
        // rate limits, CI-vector / orbital-basis caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "wave functions exceed")
            || contains_phrase_after_ident_boundary(&lower, "wave functions exceeded")
            || contains_phrase_after_ident_boundary(&lower, "wave function exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "slater determinants / slater determinant exceed(s/ed)" (FEAT-D459). Parallel to
        // `wave functions exceed` / `wave function exceed`. `slater determinant exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `slater determinants exceed` (the `s` in `determinants` prevents the singular `determinant` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `slater` so `microslater determinants exceed` / `metaslater determinants exceed` /
        // `subslater determinant exceed` do not false-positive (no space inside `slaterdeterminants`, so the
        // phrase `slater determinants exceed` is absent there; a spaced `micro slater determinants …`
        // still matches at the boundary before `slater`). `preslater determinant exceed` and
        // `reslater determinant exceed` are rejected the same way; `superslaterdeterminant exceed` has no contiguous
        // `slater determinant exceed` substring (no space between `slater` and `determinant`).
        // Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `slater determinants exceed`
        // rate limits, FCI / active-space / determinant caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "slater determinants exceed")
            || contains_phrase_after_ident_boundary(&lower, "slater determinants exceeded")
            || contains_phrase_after_ident_boundary(&lower, "slater determinant exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "configuration state functions / configuration state function exceed(s/ed)" (FEAT-D460). Parallel to
        // `slater determinants exceed` / `slater determinant exceed`. `configuration state function exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `configuration state functions exceed` (the `s` in `functions` prevents the singular `function` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `configuration` so `microconfiguration state functions exceed` / `metaconfiguration state functions exceed` /
        // `subconfiguration state function exceed` do not false-positive (no space inside `configurationstatefunctions`, so the
        // phrase `configuration state functions exceed` is absent there; a spaced `micro configuration state functions …`
        // still matches at the boundary before `configuration`). `preconfiguration state function exceed` and
        // `reconfiguration state function exceed` are rejected when `configuration` is embedded in a longer ident; `superconfiguration state function exceed`
        // does not match at the boundary before `configuration`. `superconfigurationstatefunction exceed` has no contiguous
        // `configuration state function exceed` substring.
        // Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `configuration state functions exceed`
        // rate limits, active-space / CSF-count caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "configuration state functions exceed")
            || contains_phrase_after_ident_boundary(&lower, "configuration state functions exceeded")
            || contains_phrase_after_ident_boundary(&lower, "configuration state function exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "csf coefficients / csf coefficient exceed(s/ed)" (FEAT-D461). Parallel to
        // `configuration state functions exceed` / `configuration state function exceed`. `csf coefficient exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `csf coefficients exceed` (the `s` in `coefficients` prevents the singular `coefficient` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `csf` so `microcsf coefficients exceed` / `metacsf coefficients exceed` /
        // `subcsf coefficient exceed` do not false-positive (no space inside `csfcoefficients`, so the
        // phrase `csf coefficients exceed` is absent there; a spaced `micro csf coefficients …`
        // still matches at the boundary before `csf`). `precsf coefficient exceed` and
        // `recsf coefficient exceed` are rejected when `csf` is embedded in a longer ident; `supercsf coefficient exceed`
        // does not match at the boundary before `csf`. `supercsfcoefficient exceed` has no contiguous
        // `csf coefficient exceed` substring.
        // Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `csf coefficients exceed`
        // rate limits, CI-vector / CSF-expansion caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "csf coefficients exceed")
            || contains_phrase_after_ident_boundary(&lower, "csf coefficients exceeded")
            || contains_phrase_after_ident_boundary(&lower, "csf coefficient exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "ci coefficients / ci coefficient exceed(s/ed)" (FEAT-D462). Parallel to
        // `csf coefficients exceed` / `csf coefficient exceed`. `ci coefficient exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `ci coefficients exceed` (the `s` in `coefficients` prevents the singular `coefficient` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `ci` so `microci coefficients exceed` / `metaci coefficients exceed` /
        // `subci coefficient exceed` do not false-positive (no space inside `cicoefficients`, so the
        // phrase `ci coefficients exceed` is absent there; a spaced `micro ci coefficients …`
        // still matches at the boundary before `ci`). `preci coefficient exceed` and
        // `reci coefficient exceed` are rejected when `ci` is embedded in a longer ident; `superci coefficient exceed`
        // does not match at the boundary before `ci`. `supercicoefficient exceed` has no contiguous
        // `ci coefficient exceed` substring.
        // Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `ci coefficients exceed`
        // rate limits, CI-vector / expansion caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "ci coefficients exceed")
            || contains_phrase_after_ident_boundary(&lower, "ci coefficients exceeded")
            || contains_phrase_after_ident_boundary(&lower, "ci coefficient exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "mo coefficients / mo coefficient exceed(s/ed)" (FEAT-D463). Parallel to
        // `ci coefficients exceed` / `ci coefficient exceed`. `mo coefficient exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `mo coefficients exceed` (the `s` in `coefficients` prevents the singular `coefficient` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `mo` so `micromo coefficients exceed` / `metamo coefficients exceed` /
        // `submo coefficient exceed` do not false-positive (no space inside `mocoefficients`, so the
        // phrase `mo coefficients exceed` is absent there; a spaced `micro mo coefficients …`
        // still matches at the boundary before `mo`). `premo coefficient exceed` and
        // `remo coefficient exceed` are rejected when `mo` is embedded in a longer ident; `supermo coefficient exceed`
        // does not match at the boundary before `mo`. `supermocoefficient exceed` has no contiguous
        // `mo coefficient exceed` substring.
        // Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `mo coefficients exceed`
        // rate limits, MO-basis / LCAO caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "mo coefficients exceed")
            || contains_phrase_after_ident_boundary(&lower, "mo coefficients exceeded")
            || contains_phrase_after_ident_boundary(&lower, "mo coefficient exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "natural orbitals / natural orbital exceed(s/ed)" (FEAT-D464). Parallel to
        // `molecular orbitals exceed` / `molecular orbital exceed`. `natural orbital exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `natural orbitals exceed` (the `s` in `orbitals` prevents the singular `orbital` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `natural` so `micronatural orbitals exceed` / `metanatural orbitals exceed` /
        // `subnatural orbital exceed` do not false-positive (no space inside `naturalorbitals`, so the
        // phrase `natural orbitals exceed` is absent there; a spaced `micro natural orbitals …`
        // still matches at the boundary before `natural`). `prenaturalorbital exceed` and
        // `renaturalorbital exceed` document compounds without a contiguous `natural orbital exceed` substring;
        // spaced `prenatural orbital exceed` matches the generic `orbital exceed` arm when a slot is present
        // (parallel to molecular / atomic). `supernatural orbital exceed` does not match at the boundary before `natural`.
        // Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `natural orbitals exceed`
        // rate limits, NO-truncation / occupation-number caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "natural orbitals exceed")
            || contains_phrase_after_ident_boundary(&lower, "natural orbitals exceeded")
            || contains_phrase_after_ident_boundary(&lower, "natural orbital exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "occupied orbitals / occupied orbital exceed(s/ed)" (FEAT-D467). Parallel to
        // `natural orbitals exceed` / `natural orbital exceed`. `occupied orbital exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `occupied orbitals exceed` (the `s` in `orbitals` prevents the singular `orbital` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `occupied` so `microoccupied orbitals exceed` / `metaoccupied orbitals exceed` /
        // `suboccupied orbital exceed` do not false-positive (no space inside `occupiedorbitals`, so the
        // phrase `occupied orbitals exceed` is absent there; a spaced `micro occupied orbitals …`
        // still matches at the boundary before `occupied`). `preoccupied orbitals exceed` and
        // `unoccupied orbitals exceed` embed `occupied` without a boundary before the token.
        // Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `occupied orbitals exceed`
        // rate limits, frozen-core / active-space caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "occupied orbitals exceed")
            || contains_phrase_after_ident_boundary(&lower, "occupied orbitals exceeded")
            || contains_phrase_after_ident_boundary(&lower, "occupied orbital exceed"))
            && explicit_context_slot_after_ident_boundary(&lower))
        // Plural / singular "canonical orbitals / canonical orbital exceed(s/ed)" (FEAT-D465). Parallel to
        // `natural orbitals exceed` / `natural orbital exceed`. `canonical orbital exceed` matches present/past via
        // `exceed` prefix of `exceeds` / `exceeded` and does not substring-match plural
        // `canonical orbitals exceed` (the `s` in `orbitals` prevents the singular `orbital` + space +
        // `exceed` path from aligning inside the plural phrase).
        // Ident-boundary at `canonical` so `microcanonical orbitals exceed` / `metacanonical orbitals exceed` /
        // `subcanonical orbital exceed` do not false-positive (no space inside `canonicalorbitals`, so the
        // phrase `canonical orbitals exceed` is absent there; a spaced `micro canonical orbitals …`
        // still matches at the boundary before `canonical`). `precanonicalorbital exceed` and
        // `recanonicalorbital exceed` document compounds without a contiguous `canonical orbital exceed` substring;
        // spaced `precanonical orbital exceed` matches the generic `orbital exceed` arm when a slot is present.
        // `microcanonical` as one token before `orbitals` does not match at the boundary before `canonical`.
        // Same explicit context-slot phrases as `messages exceed`. Negatives: HTTP `canonical orbitals exceed`
        // rate limits, CASSCF / active-space caps, etc. without slot wording.
        || ((contains_phrase_after_ident_boundary(&lower, "canonical orbitals exceed")
            || contains_phrase_after_ident_boundary(&lower, "canonical orbitals exceeded")
            || contains_phrase_after_ident_boundary(&lower, "canonical orbital exceed"))
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

#[inline]
fn tool_result_truncation_eligible(index: usize, msg: &crate::ollama::ChatMessage) -> bool {
    !(index == 0 && msg.role == "system")
}

/// Token estimate per message — keep aligned with `context_assembler::estimate_message_tokens`
/// (cannot import: `context_assembler` depends on this module for `CHARS_PER_TOKEN`).
fn estimate_message_tokens_for_budget(m: &crate::ollama::ChatMessage) -> usize {
    let mut n = m.content.chars().count() / CHARS_PER_TOKEN;
    if let Some(imgs) = m.images.as_ref() {
        for b64 in imgs {
            n = n.saturating_add(b64.len() / CHARS_PER_TOKEN);
        }
    }
    n.saturating_add(4)
}

fn estimate_messages_token_total_local(messages: &[crate::ollama::ChatMessage]) -> usize {
    messages
        .iter()
        .map(estimate_message_tokens_for_budget)
        .sum()
}

#[inline]
fn context_token_budget_local(context_size_tokens: u32) -> usize {
    context_size_tokens
        .saturating_sub(PROACTIVE_CTX_SAFETY_TOKENS)
        .max(256) as usize
}

#[derive(Clone, Copy)]
enum ToolResultTruncationReason {
    ContextOverflowRetry,
    ProactiveContextBudget,
}

/// Truncate one message to at most `max_chars` body (word/line boundary), with a reason-specific
/// trailer. Returns whether the message was changed.
fn apply_tool_result_truncate(
    msg: &mut crate::ollama::ChatMessage,
    max_chars: usize,
    reason: ToolResultTruncationReason,
) -> bool {
    let char_count = msg.content.chars().count();
    if char_count <= max_chars {
        return false;
    }
    let truncated_body = truncate_at_boundary(&msg.content, max_chars);
    let trimmed = truncated_body.trim_end();
    let suffix = match reason {
        ToolResultTruncationReason::ContextOverflowRetry => format!(
            "\n\n[truncated from {} to {} chars due to context limit]",
            char_count, max_chars
        ),
        ToolResultTruncationReason::ProactiveContextBudget => format!(
            "\n\n[compacted proactively for context budget: {} → {} chars]",
            char_count, max_chars
        ),
    };
    msg.content = format!("{trimmed}{suffix}");
    true
}

/// Inner logic for tests and tuning; production uses [`proactively_compact_tool_results_for_context_budget`].
pub(crate) fn proactively_compact_tool_results_for_context_budget_impl(
    messages: &mut [crate::ollama::ChatMessage],
    context_size_tokens: u32,
    headroom_ratio: f64,
    max_chars: usize,
) -> usize {
    let budget_tokens = context_token_budget_local(context_size_tokens);
    let headroom = headroom_ratio.clamp(0.05_f64, 0.45_f64);
    let threshold = ((budget_tokens as f64) * (1.0_f64 - headroom)).floor() as usize;
    let threshold = threshold.max(256);
    const MIN_BODY: usize = 256;
    let mut total = 0usize;

    for _ in 0..64 {
        let est = estimate_messages_token_total_local(messages);
        if est <= threshold {
            break;
        }
        // Phase A — oldest message still over `max_chars` (typical huge FETCH/BROWSER payload).
        let mut idx_phase_a: Option<usize> = None;
        for (i, msg) in messages.iter().enumerate() {
            if !tool_result_truncation_eligible(i, msg) {
                continue;
            }
            if msg.content.chars().count() > max_chars {
                idx_phase_a = Some(match idx_phase_a {
                    None => i,
                    Some(j) => j.min(i),
                });
            }
        }
        if let Some(i) = idx_phase_a {
            if apply_tool_result_truncate(
                &mut messages[i],
                max_chars,
                ToolResultTruncationReason::ProactiveContextBudget,
            ) {
                tracing::info!(
                    target: "mac_stats::context_budget",
                    "Proactive context budget: compacted message index {} to {} chars (est_tokens={} threshold={})",
                    i,
                    max_chars,
                    est,
                    threshold
                );
                total += 1;
            }
            continue;
        }
        // Phase B — no message above `max_chars`; shave oldest eligible body still above MIN_BODY.
        let mut pick: Option<usize> = None;
        for (i, msg) in messages.iter().enumerate() {
            if !tool_result_truncation_eligible(i, msg) {
                continue;
            }
            let c = msg.content.chars().count();
            if c <= MIN_BODY {
                continue;
            }
            pick = Some(match pick {
                None => i,
                Some(j) => j.min(i),
            });
        }
        let Some(i) = pick else {
            tracing::debug!(
                target: "mac_stats::context_budget",
                "Proactive context budget: still over threshold (est_tokens={} > {}) but no further eligible messages to shrink",
                est,
                threshold
            );
            break;
        };
        let c = messages[i].content.chars().count();
        let target = (c * 4 / 5).max(MIN_BODY).min(c.saturating_sub(1));
        if target >= c {
            break;
        }
        if apply_tool_result_truncate(
            &mut messages[i],
            target,
            ToolResultTruncationReason::ProactiveContextBudget,
        ) {
            tracing::info!(
                target: "mac_stats::context_budget",
                "Proactive context budget: shaved message index {} to {} chars (est_tokens={} threshold={})",
                i,
                target,
                est,
                threshold
            );
            total += 1;
        }
    }
    total
}

/// Before an Ollama chat call, optionally compact tool-style messages when the estimated token
/// total exceeds a headroom threshold below the model context budget (OpenClaw-style proactive
/// guard). Reactive overflow truncation remains as a safety net.
pub(crate) fn proactively_compact_tool_results_for_context_budget(
    messages: &mut [crate::ollama::ChatMessage],
    context_size_tokens: u32,
) -> usize {
    if !crate::config::Config::proactive_tool_result_context_budget_enabled() {
        return 0;
    }
    if !crate::config::Config::context_overflow_truncate_enabled() {
        return 0;
    }
    let n = proactively_compact_tool_results_for_context_budget_impl(
        messages,
        context_size_tokens,
        crate::config::Config::proactive_context_budget_headroom_ratio(),
        crate::config::Config::proactive_context_max_result_chars(),
    );
    if n > 0 {
        tracing::info!(
            target: "mac_stats::context_budget",
            "Proactive tool-result context budget: completed {} compaction step(s)",
            n
        );
    }
    n
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
        if !tool_result_truncation_eligible(i, msg) {
            continue;
        }
        if apply_tool_result_truncate(
            msg,
            max_chars_per_result,
            ToolResultTruncationReason::ContextOverflowRetry,
        ) {
            truncated_count += 1;
        }
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
    ollama_http_queue: OllamaHttpQueue,
) -> Result<String, String> {
    use tracing::info;
    let messages = vec![
        crate::ollama::ChatMessage {
            role: "system".to_string(),
            content: skill_content.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
        },
        crate::ollama::ChatMessage {
            role: "user".to_string(),
            content: user_message.to_string(),
            images: None,
            tool_calls: None,
            tool_name: None,
            tool_call_id: None,
        },
    ];
    info!(
        "Agent router: SKILL session request (user message {} chars)",
        user_message.chars().count()
    );
    let response = send_ollama_chat_messages(
        messages,
        model_override,
        options_override,
        ollama_http_queue,
    )
    .await?;
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
    let mut cmd = Command::new("node");
    crate::security::host_exec_env::apply_host_exec_env_hardening(&mut cmd);
    let out = cmd
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
mod tests;
