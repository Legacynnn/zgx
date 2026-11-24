---
section: fixed
---
- **A scroll view that shrinks as it scrolls leaves nothing of what it showed behind** (zgx): where its box was and isn't now repaints. Its content's damage was taken where the content had moved to, so a row painted below the view's new edge stayed on screen, whether the scroll copied or jumped.
