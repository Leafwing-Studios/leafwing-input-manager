# Feature Name: (fill me in with a unique ident, `my_awesome_feature`)

## Summary

One paragraph explanation of the feature.

## Motivation

Why are we doing this? What use cases does it support, which problems does it solve or what experience does it enable?

## User-facing explanation

Explain the proposal as if it was already included and you were teaching it to a user or player. That generally means:

- Introducing new named concepts.
- Explaining the feature, ideally through simple examples of solutions to concrete problems.
- Explaining how users should *think* about the feature, and how it should impact the way they use our code or game. It should explain the impact as concretely as possible.
- If applicable, explain how this feature compares to similar existing features, and in what situations the user would use each one.

## \[Optional\] Game design analysis

### Choices

- What choices can players make?
  - What impact will those choices have?
  - Why would different players choose to make different choices?
  - Why would the same player choose to make different choices?
- What are the core tensions present in this design?

### Feedback loops

What feedback loops does this design create or build upon?

### Tuning levers

What values or elements of the design can we trivially change after the feature is built to alter power, feel or complexity?

### Hooks

- How can the rest of the game interact with this system?
- How can this system interact with the rest of the game?

## Implementation strategy

This is the technical portion of the RFC.
Try to capture the broad implementation strategy,
and then focus in on the tricky details so that:

- Its interaction with other features is clear.
- It is reasonably clear how the feature would be implemented.
- Corner cases are dissected by example.

When necessary, this section should return to the examples given in the previous section and explain the implementation details that make them work.

When writing this section be mindful of the following:

- **RFCs should be scoped:** Try to avoid creating RFCs for huge design spaces that span many features. Try to pick a specific feature slice and describe it in as much detail as possible. Feel free to create multiple RFCs if you need multiple features.
- **RFCs should avoid ambiguity:** Two developers implementing the same RFC should come up with nearly identical implementations.
- **RFCs should be "implementable":** Merged RFCs should only depend on features from other merged RFCs and existing features. It is ok to create multiple dependent RFCs, but they should either be merged at the same time or have a clear merge order that ensures the "implementable" rule is respected.

## Drawbacks

- Why should we *not* do this?
- Which technical constraints are we pushing up against?
- Which design constraints are we pushing up against?
- Which product area (art, sound, programming, design, narrative etc.) will this feature tax the most?

## Rationale and alternatives

- Why is this design the best in the space of possible designs?
- What other designs have been considered and what is the rationale for not choosing them?
- What objections immediately spring to mind? How have you addressed them?
- What is the impact of not doing this?

## \[Optional\] Prior art

Discuss prior art, both the good and the bad, in relation to this proposal.
This can include:

- Does this feature exist in other libraries and what experiences have their community had?
- Papers: Are there any published papers or great posts that discuss this?

This section is intended to encourage you as an author to think about the lessons from other tools and provide readers of your RFC with a fuller picture.

Note that while precedent set by other engines is some motivation, it does not on its own motivate an RFC.

## Unresolved questions

- What parts of the design do you expect to resolve through the RFC process before this gets merged?
- What parts of the design do you expect to resolve through the implementation of this feature before the feature PR is merged?
- What related issues do you consider out of scope for this RFC that could be addressed in the future independently of the solution that comes out of this RFC?
- Which questions need to be answered through experimentation before we can commit to this feature?
  - What hypotheses do we have?
  - What is the simplest, most focused ways we can test each hypothesis?

## \[Optional\] Future possibilities

Think about what the natural extension and evolution of your proposal would
be and how it would affect Bevy as a whole in a holistic way.
Try to use this section as a tool to more fully consider other possible
interactions with the engine in your proposal.

This is also a good place to "dump ideas", if they are out of scope for the
RFC you are writing but otherwise related.

Note that having something written down in the future-possibilities section
is not a reason to accept the current or a future RFC; such notes should be
in the section on motivation or rationale in this or subsequent RFCs.
If a feature or change has no direct value on its own, expand your RFC to include the first valuable feature that would build on it.
