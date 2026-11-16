// Single source for the project's public URLs, so every section links to the
// same repository pages and queries the same release API.

export const GITHUB_REPO_URL = 'https://github.com/eatbas/insomniapp'
export const GITHUB_RELEASES_URL = `${GITHUB_REPO_URL}/releases`
export const GITHUB_LATEST_RELEASE_URL = `${GITHUB_RELEASES_URL}/latest`
export const GITHUB_ISSUES_URL = `${GITHUB_REPO_URL}/issues`
export const GITHUB_LATEST_RELEASE_API_URL =
  'https://api.github.com/repos/eatbas/insomniapp/releases/latest'
