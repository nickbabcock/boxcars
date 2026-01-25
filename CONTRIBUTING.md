## Contributing to boxcars

Rocket League patches often introduce new fields that cause parsing to fail with an "attribute unknown or not implemented" error.

Pull requests that update the attribute mapping in `src/data.rs` **must** include a sample replay in the test suite to prevent future regressions. This project uses [insta](https://insta.rs/) for snapshot testing.

How to add a test case:

1. Place the `.replay` file in `assets/replays/good/`. Use a short (4–5 chars) or descriptive filename.
2. Run the tests with the update flag to automatically create the new snapshot files (alternatively use cargo-insta or vscode):
  ```sh
  INSTA_UPDATE=always cargo test
  ```
3. Include both the `.replay` file and the generated `.snap` files in the pull request
