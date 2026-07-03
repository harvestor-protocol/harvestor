# Harvestor + Stellar Wave Program Setup

This document explains how to launch Harvestor on the Stellar Wave Program and manage the contributor community.

## What You've Got

### 1. CONTRIBUTING.md (258 lines)
Complete guide for developers joining the project:
- Local environment setup (Rust, Soroban CLI, Node, PostgreSQL)
- How to run tests and build contracts
- Git conventions and commit message style
- PR process and code review expectations
- Issue complexity tiers aligned with Stellar Wave points
- Welcoming note for bounty hunters

**File**: [CONTRIBUTING.md](./CONTRIBUTING.md)

### 2. GITHUB_ISSUES.md (724 lines)
Pre-formatted backlog of 8 issues, ready to copy into GitHub:

| # | Title | Complexity | Points | Effort | Type |
|---|-------|-----------|--------|--------|------|
| 1 | Score boundary validation tests | Trivial | 100 | 1–2 hrs | test |
| 2 | Add missing doc comments | Trivial | 100 | 1–2 hrs | docs |
| 3 | Rate-limiting for scores | Medium | 150 | 4–8 hrs | feature |
| 4 | Cross-contract integration tests | Medium | 150 | 4–8 hrs | test |
| 5 | Repayment edge case handling | Medium | 150 | 4–8 hrs | feature |
| 6 | Events/logging system | High | 200 | 16+ hrs | feature |
| 7 | GitHub Actions CI/CD | High | 200 | 16+ hrs | chore |
| 8 | Good first issue walkthrough | Medium | 150 | 4–8 hrs | docs |

**Total**: 1,100 points available (6 issues × 150 + 2 issues × 100 + 2 issues × 200)

**File**: [GITHUB_ISSUES.md](./GITHUB_ISSUES.md)

## How to Launch on Stellar Wave

### Step 1: Set Up GitHub Issues

1. Create a new GitHub issue for each item in `GITHUB_ISSUES.md`
2. Copy the **Body** content into the issue description
3. Apply the **Labels** (listed in each issue)
4. Optionally add a **Milestone** (e.g., "v1.0")
5. Leave issues **unassigned** (contributors will claim them)

### Step 2: Update Repository Settings

1. Go to **GitHub Settings → Features → Discussions** and enable them
2. Set **Default branch** to `main`
3. Require PR reviews before merge (recommended)
4. Enable branch protection on `main`:
   - Require PR reviews (at least 1)
   - Require status checks to pass (CI/CD)
   - Dismiss stale reviews when new commits pushed

### Step 3: Configure Stellar Wave

1. Register your GitHub repository with Stellar Wave Program
2. Specify point values:
   - Trivial = 100 pts
   - Medium = 150 pts
   - High = 200 pts
3. Connect your GitHub account for payment/rewards management

### Step 4: Invite Contributors

Share these links:
- **Repository**: https://github.com/harvestor-protocol/harvestor
- **Issues**: https://github.com/harvestor-protocol/harvestor/issues
- **Contributing Guide**: [CONTRIBUTING.md](./CONTRIBUTING.md)

Example message:
> "Harvestor is now open for contributions via the Stellar Wave Program. 
> Check out the issues and CONTRIBUTING.md to get started. We're looking 
> for real contributors, not just bounty farmers. This is a long-term project 
> with genuine impact."

## Managing Contributors

### For Issue Reporters (You)

When a contributor comments "I'd like to work on this!":
1. Verify the issue is clear and well-scoped
2. Assign the issue to them (GitHub: Assignees → Add)
3. Optionally set a due date (e.g., 2 weeks)

When a PR is submitted:
1. Review for:
   - Tests pass (CI/CD green)
   - Code follows conventions (CONTRIBUTING.md)
   - Doc comments present
   - Scope matches the issue
2. Request changes or approve
3. Merge when approved and conflicts resolved

### For Contributors

They follow the process in CONTRIBUTING.md:
1. Pick an issue labeled `good-first-issue` or any issue they like
2. Comment "I'd like to work on this"
3. Create a branch: `feature/description`
4. Implement, test, add doc comments
5. Commit with proper message format
6. Push and open a PR
7. Address review feedback
8. PR merged → Stellar Wave award points

## Communication Channels

Set these up to help contributors:

1. **GitHub Discussions**: 
   - Create category "Getting Started" for questions
   - Create category "Architecture" for design discussions

2. **Stellar Community**:
   - Post in [Stellar Developers Slack](https://stellar-slack.herokuapp.com/) 
   - Invite contributors to the project channel

3. **Email** (optional):
   - Point CONTRIBUTING.md to a real email address
   - Or use GitHub Issues for all communication

## Timeline Suggestion

- **Week 1**: Create issues, enable Stellar Wave integration
- **Week 2–3**: First contributors pick issues
- **Week 4–6**: First PRs reviewed and merged
- **Month 2+**: Iterate on roadmap, add new issues

## Key Success Factors

✅ **Clear scoping**: Each issue has acceptance criteria (not vague)  
✅ **Realistic effort**: Trivial issues actually take 1–2 hours  
✅ **Good first issues**: Issue #1 and #2 are explicitly tagged for newcomers  
✅ **Active reviews**: Respond to PRs within 24–48 hours  
✅ **Welcoming tone**: CONTRIBUTING.md invites Stellar Wave contributors  
✅ **Real project**: Issues align with genuine roadmap, not make-work  

## Checklist for Launch

- [ ] CONTRIBUTING.md reviewed and finalized
- [ ] GITHUB_ISSUES.md reviewed, issues created on GitHub
- [ ] GitHub issues labeled with `complexity:trivial/medium/high`
- [ ] Issues #1 and #2 tagged `good-first-issue`
- [ ] Branch protection rules enabled on `main`
- [ ] Stellar Wave Program integration set up
- [ ] README.md links to CONTRIBUTING.md
- [ ] Repository README updated with "Contributing" section
- [ ] Slack/community channels notified
- [ ] First batch of issues published

## Next Steps

Once contributors start submitting PRs:
1. Review and merge (or request changes)
2. Thank them in the PR comments
3. Watch for patterns (which issues are popular, which are blocked)
4. Add follow-up issues based on feedback

Good luck with Harvestor on Stellar Wave!
