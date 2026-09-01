allm is intentionally lean. Core guidelines:

- Keep the codebase clean. Don't overcomplicate things, don't write 100 lines for something that can be done in 20. 
- Code should be centralized and readable. Don't create a separate file if there's already a file that's related to it.
- Keep code comments short and simple, should explain the why, NOT the what and the how. Don't split comments, keep them on one line.

When committing & git stuff:
- Don't commit tests. You're fine to add tests, but please remove them before finishing up. Have faith in your code! Testing is the human's job at runtime.
- Don't write PRs yourself, get the human to do it. It's a show of understanding.
- Commit names should be short and succinct, descriptions should be readable, try not to be verbose, 10 lines max.
