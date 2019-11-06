# Contributing to Dapr Rust SDK

## Fork and set upstream

From the GitHub project at [https://github.com/dapr/rust-sdk](https://github.com/dapr/rust-sdk), click `fork` to create your own copy of the repository.  

Clone your fork of the code:

``` bash
git clone https://github.com/<USER-NAME>/rust-sdk
```

Change directory into the project with `cd rust-sdk`.

From the root of the project, set the `upstream` branch to the original project repository:

``` bash
git remote add upstream https://github.com/dapr/rust-sdk
```

This will allow you to rebase your fork ontop of the changes pushed to the original.

## Updating your fork

As your work progresses, it is helpful to regularly update your fork of the code to replay the work you have done on top of the latest code pushed to the original repository. In the above section, you set the `upstream` repository to the original code. Now, you can ensure you are always working with the latest code by pulling it in to your fork.

Fetch the latest updates to the original repository:

``` bash
git fetch upstream
```

If you want to rebase your branch ontop of the original repository's `master` branch:

``` bash
git rebase upstream/master
```

This will replay the changes you have made in your branch on top of the upstream `master` branch. The same principle applies to other branches in the original repository as well.
