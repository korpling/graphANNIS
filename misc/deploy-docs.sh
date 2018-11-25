#!/bin/bash

# Used by CI to deploy the existing documentation/book/<version> directory to Github Pages

if [ -n "$GITHUB_API_KEY" ]; then
    cd "$TRAVIS_BUILD_DIR"

    echo "cloning gh-pages"
    git clone  --single-branch gh-pages https://thomaskrause:$GITHUB_API_KEY@github.com/corpus-tools/graphANNIS gh-pages
    cp -R documentation/book/* gh-pages/
    cd gh-pages
    git add .
    git -c user.name='travis' -c user.email='travis' commit -m "update documentation"
    echo "pushing to gh-pages"
    git push -q https://thomaskrause:$GITHUB_API_KEY@github.com/corpus-tools/graphANNIS gh-pages &>/dev/null
    cd "$TRAVIS_BUILD_DIR"
fi