#!/bin/bash

set -e

GITHUB_USER="doc-deploy-bot"
GITHUB_EMAIL=""

DOC_VERSION=${TRAVIS_TAG:-$TRAVIS_BRANCH}
echo "Deploying documentation version $DOC_VERSION"

if [ -d "target/doc" ]; then
  echo "<meta http-equiv=refresh content=0;url=parquet/index.html>" > ./target/doc/index.html

  echo "Setting up gh-pages branch" &&
  git clone --branch gh-pages "https://$GITHUB_TOKEN@github.com/${TRAVIS_REPO_SLUG}.git" deploy_docs > /dev/null 2>&1 &&
  cd deploy_docs &&
  rm -rf ./$DOC_VERSION &&
  mkdir ./$DOC_VERSION &&
  mv ../target/doc/* ./$DOC_VERSION/ &&
  git config user.name "$GITHUB_USER" &&
  git config user.email "$GITHUB_EMAIL" &&
  git add -A . &&
  git commit -m "Deploy doc pages $DOC_VERSION at ${TRAVIS_COMMIT}" &&
  echo "Pushing documentation update" &&
  git push --quiet origin gh-pages > /dev/null 2>&1 &&
  echo "Published documentation $DOC_VERSION" || echo "Failed to publish documentation $DOC_VERSION"
else
  echo "Failed to find target/doc directory"
  exit 1
fi
