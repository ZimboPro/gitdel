# Git Del

A small utility to make it simple to delete local git branches if the remote branches are no longer available.

The Bash command equivalent is that the utility is based on:
```bash
git fetch --prune && git branch -r | awk "{print \$1}" | egrep -v -f /dev/fd/0 <(git branch -vv | grep origin) | awk "{print \$1}" | xargs git branch -d
```
