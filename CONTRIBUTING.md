# Contributing to SAYT

SAYT is in the M0 scaffold stage. Feature work should follow the milestone gates in [`docs/roadmap/milestones.md`](docs/roadmap/milestones.md) and the architecture decisions in [`docs/architecture/adr/`](docs/architecture/adr/).

## Before opening a change

1. Keep the change within the current milestone, or document why it must move a gate.
2. Add or update documentation with behavior changes.
3. Add tests that exercise the new behavior and at least one relevant failure or boundary case.
4. Run the checks in the README and include their results in the pull request.
5. Do not add copyrighted music fixtures, provider-private endpoints, tokens, credentials, or user library data.

## Developer Certificate of Origin

SAYT proposes the Developer Certificate of Origin (DCO) sign-off model and no Contributor License Agreement. The project owner must confirm this policy before accepting outside contributions. If confirmed, contributors sign each commit with `git commit -s`; the sign-off attests to the DCO text at <https://developercertificate.org/>.

## Pull requests

Use the repository pull request template. State the milestone, observable behavior, evidence, license changes, and the production failure you considered. Do not claim a controller, decoder, device, or performance target is supported without its documented verification evidence.
