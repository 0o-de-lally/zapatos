# Zap Manifesto

Zap is a minimalist version of aptos, that has 99% of the features and 5% of the code. There are 6 years of legacy code in aptos. We want to start over. The new create will IMPORT NOTHING from zapatos or other crates.

## Core Rules

1.  **Zero Start**: It will start with zero code.
2.  **Canonical Copy**: It will copy canonical files from the wider zapatos project, and will strip down the files and modules to their essntial functions.
3.  **No Features**: rule #1 there are no compiler features, there is just DEFAULT.
4.  **No Fuzzing**: fuzzing is not necessary.
5.  **Env Var Logs**: diem move prints should be gated by ENV vars not features.
6.  **Import Tests**: For every file imported and function recreated we need to also bring the relevant tests.

## Milestone 1

Make a fullnode binary connect and sync with aptos network.
