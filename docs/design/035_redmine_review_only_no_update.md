# Redmine review/summarize-only: do not update or close the ticket

## Problem

When the user asked only to **review** or **summarize** a Redmine ticket (e.g. "Summarize or review redmine ticket 7209"), the system over-applied success criteria and nudged the model to update or close the ticket. That led to:

1. **Success criteria extractor** returning full "close ticket" criteria (e.g. "ticket marked as resolved; all subtasks completed; QA approved") instead of "summary provided".
2. **Verifier** comparing the model's summary to those inflated criteria → answering NO.
3. **Retry** with "Complete the remaining steps" → model then did REDMINE_API PUT (status In Progress / Resolved), changing the ticket even though the user only asked for a review.

## Fix (implemented in `ollama.rs`)

1. **Override success criteria for review/summarize-only**  
   `is_redmine_review_or_summarize_only(question)` detects: question has a Redmine/ticket/issue ID + "review" or "summarize", and does *not* mention update, add comment, close, resolve, or "write".  
   When true, we set success criteria to a single item: *"Summary of ticket content provided to the user."* (no extraction from the LLM for that case).

2. **Redmine GET result: no PUT hint for review-only**  
   After a REDMINE_API GET, we used to always append: "If the user asked to **update** this ticket or **add a comment**, your next reply MUST be exactly one line: REDMINE_API: PUT …".  
   For review/summarize-only we now append instead: "The user asked only to review/summarize. Do NOT update the ticket or add a comment. Reply with your summary and DONE: success."

3. **Retry guard**  
   When verification says "not satisfied" and we are about to retry: if the **original** question was review/summarize-only and the response already contains a ticket summary (length > 150 and keywords like Subject, Description, Status, Redmine), we do **not** send "Complete the remaining steps". We send: "The request was only to review/summarize. A summary was already provided. Reply with a brief confirmation and DONE: success; do not update or close the ticket."  
   So the retry confirms success instead of pushing an update/close.

---

## Intermediate answer on Discord (thought process)

When verification says "not satisfied" and we retry (Discord path), we now surface the thought process to the user:

- The reply is formatted as: **--- Intermediate answer:** … **---** (the first-pass reply), then either **--- Final answer:** … **---** or **Final answer is the same as intermediate.**
- So the user sees what we had after the first pass and whether the retry added anything. If the retry only confirmed (e.g. review-only case), we avoid repeating the blob and just say "Final answer is the same as intermediate."
- Implemented in `answer_with_ollama_and_fetch`: parameter `discord_intermediate` is set when recursing from the retry branch; on return we compare final vs intermediate and format the single Discord message accordingly.

---

## See also

- `docs/033_redmine_review_hallucination_fix.md` — why Redmine review sometimes hallucinated (different code path / missing ticket data).
