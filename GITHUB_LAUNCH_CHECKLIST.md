# GitHub Launch Checklist for Harvestor

## Step 1: Prepare Your GitHub Repository

- [ ] Ensure your GitHub repo exists at: `https://github.com/harvestor-protocol/harvestor`
- [ ] Make sure you have admin/write access to the repository
- [ ] (Optional) Enable GitHub Discussions for community support

## Step 2: Create the 8 GitHub Issues

Open `CREATE_ISSUES_MANUALLY.md` in your repository and follow the steps:

**Quick Reference**:
1. Go to `https://github.com/harvestor-protocol/harvestor/issues`
2. Click **New issue**
3. Copy each issue title, labels, and body from `CREATE_ISSUES_MANUALLY.md`
4. Create each issue (takes ~15 minutes total)

**Issues to create** (in order):
- [ ] Issue 1: Score boundary validation tests (trivial, 100 pts)
- [ ] Issue 2: Doc comments (trivial, 100 pts)
- [ ] Issue 3: Rate-limiting (medium, 150 pts)
- [ ] Issue 4: Integration tests (medium, 150 pts)
- [ ] Issue 5: Repayment edge cases (medium, 150 pts)
- [ ] Issue 6: Events system (high, 200 pts)
- [ ] Issue 7: CI/CD pipeline (high, 200 pts)
- [ ] Issue 8: Walkthrough guide (medium, 150 pts)

## Step 3: Configure GitHub Repository Settings

### Branch Protection

- [ ] Go to **Settings** → **Branches** → **Branch protection rules**
- [ ] Add rule for `main` branch:
  - Require pull request reviews before merging (1 review minimum)
  - Require status checks to pass before merging (once CI is set up)
  - Dismiss stale pull request approvals when new commits are pushed
  - Require branches to be up to date before merging

### Labels

- [ ] Go to **Issues** → **Labels**
- [ ] Create these labels if they don't exist:
  - `type:test` – Testing and QA
  - `type:feature` – New feature
  - `type:docs` – Documentation
  - `type:ci` – CI/CD and infrastructure
  - `complexity:trivial` – 1–2 hours, good for beginners
  - `complexity:medium` – 4–8 hours, intermediate
  - `complexity:high` – 16+ hours, advanced
  - `good-first-issue` – Recommended for newcomers
  - `help-wanted` – Contributors needed
  - `bug` – Bug fix

## Step 4: Update Repository Documentation

- [ ] README.md links to CONTRIBUTING.md (already done)
- [ ] README.md points to CREATE_ISSUES_MANUALLY.md or Issues tab
- [ ] Add link to QUICKSTART.md for setup instructions
- [ ] Consider pinning an issue or discussion about Stellar Wave Program

## Step 5: Register with Stellar Wave Program

- [ ] Go to [Stellar Wave Program](https://stellar.org/grants-and-funding)
- [ ] Register your project and issues
- [ ] Set point values and bounty amounts:
  - Trivial (100 pts): $100–$200
  - Medium (150 pts): $200–$400
  - High (200 pts): $400–$800
  - Adjust based on your budget
- [ ] Set up payment method (Stripe, direct transfer, or via Stellar Wave)
- [ ] Share registration link in your community

## Step 6: Promote Your Project

- [ ] Post in [Stellar Developers Slack](https://stellar-slack.herokuapp.com/)
  - Link to README.md
  - Link to GitHub Issues
  - Mention Stellar Wave bounties
- [ ] Post in [Stellar Community Discord](https://discord.gg/stellardev)
- [ ] Create a GitHub Discussion: "Welcome contributors!"
  - Introduce Harvestor
  - Link to CONTRIBUTING.md
  - Link to good-first-issues
  - Mention Stellar Wave Program
- [ ] (Optional) Post on Twitter/X with Stellar hashtags

## Step 7: Respond to Contributors

When contributors express interest:

- [ ] Assign the issue to them
- [ ] Link to CONTRIBUTING.md if they ask questions
- [ ] Review PRs carefully (use GitHub PR review tools)
- [ ] Provide constructive feedback
- [ ] Merge when tests pass and code is good
- [ ] Approve bounty payments via Stellar Wave or your payment method

## Example Communication

> Thanks for your interest! Here's how to get started:
> 
> 1. Read [CONTRIBUTING.md](../CONTRIBUTING.md) for setup
> 2. Check [QUICKSTART.md](../QUICKSTART.md) for building contracts
> 3. Create a branch: `git checkout -b fix/score-boundary-tests`
> 4. Implement and test: `cargo test --lib`
> 5. Commit and push
> 6. Open a PR
> 
> For Stellar Wave bounty info, see [Stellar Wave Program](https://stellar.org/grants-and-funding)
> 
> Questions? Ask here or in [Stellar Developers Slack](https://stellar-slack.herokuapp.com/)

## Tracking Progress

- [ ] Set up GitHub Projects board (optional but recommended)
  - Columns: Backlog, In Progress, Review, Done
  - Automatically move issues when PRs are created/merged
- [ ] Weekly: Check for inactive issues or stalled PRs
- [ ] Monthly: Report progress to Stellar Wave Program (if required)

## Success Metrics

- [ ] At least 1 PR submitted within first week
- [ ] At least 50% of issues completed within 4 weeks
- [ ] At least 3 unique contributors
- [ ] Positive feedback from contributors
- [ ] Code quality maintained (all CI checks passing)

## Support & Next Steps

If something goes wrong:
- Check [CONTRIBUTING.md](./CONTRIBUTING.md) troubleshooting section
- Ask in [Stellar Developers Slack](https://stellar-slack.herokuapp.com/)
- Open an issue on the Harvestor repository itself
- Contact Stellar Wave Program support

---

**Status**: Ready to launch!

**Questions?** Start with README.md, then CONTRIBUTING.md, then STELLAR_WAVE_SETUP.md.
