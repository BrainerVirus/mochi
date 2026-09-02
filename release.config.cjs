module.exports = {
  branches: ["main"],
  tagFormat: "v${version}",
  plugins: [
    // Must be first (workit AR-16: @semantic-release/exec v7 renamed
    // analyzeCmd -> analyzeCommitsCmd; old name is silently ignored).
    // Printing nothing skips the release entirely — no tag, no publish.
    [
      "@semantic-release/exec",
      {
        analyzeCommitsCmd: "node scripts/release/analyze-release-scope.mjs",
      },
    ],
    "@semantic-release/release-notes-generator",
    "@semantic-release/github",
  ],
};
