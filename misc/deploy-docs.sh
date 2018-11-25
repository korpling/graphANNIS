#!/bin/bash

# Used by CI to deploy the existing documentation/book/<version> directory to Github Pages

if [ -n "$GITHUB_API_KEY" ]; then
    cd "$TRAVIS_BUILD_DIR/documentation/book"
    git init
    git checkout -b gh-pages
    git add ${TRAVIS_TAG}
    git -c user.name='travis' -c user.email='travis' commit -m documentation
    git push -q https://thomaskrause:$GITHUB_API_KEY@github.com/corpus-tools/graphANNIS-gh-pages gh-pages &>/dev/null
    cd "$TRAVIS_BUILD_DIR"
fi