#!/bin/bash

# Used by CI to deploy the existing documentation/book/<version> directory to Github Pages

if [ -n "$GITHUB_API_KEY" ]; then
    cd "$TRAVIS_BUILD_DIR"

    echo "cloning gh-pages"
    git clone -q  -b gh-pages https://thomaskrause:$GITHUB_API_KEY@github.com/corpus-tools/graphANNIS gh-pages &>/dev/null
    cd gh-pages
    mkdir -f docs
    cp -R ../docs/book/* gh-pages/
    git add .
    git -c user.name='travis' -c user.email='travis' commit -m "update documentation"
    echo "pushing to gh-pages"
    git push -q https://thomaskrause:$GITHUB_API_KEY@github.com/corpus-tools/graphANNIS gh-pages &>/dev/null
    cd "$TRAVIS_BUILD_DIR"
fi