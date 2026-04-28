#!/bin/bash
set -e

# Build locally → upload dist/ as a static site to the "dist" Vercel project.
# The project link is stashed at .vercel-link/ (outside dist/) because vite
# clears dist/ on every build.

REPO=$(cd "$(dirname "$0")" && pwd)
cd "$REPO/frontend"

echo "Building locally..."
npm run build
cp vercel.json dist/

echo "Restoring Vercel project link..."
mkdir -p dist/.vercel
cp "$REPO/.vercel-link/project.json" dist/.vercel/project.json

echo "Deploying static dist/ to Vercel..."
cd dist
LOG=$(mktemp)
npx vercel@latest deploy --prod --yes 2>&1 | tee "$LOG"
DEPLOY_URL=$(grep -oE 'https://[a-zA-Z0-9.-]+\.vercel\.app' "$LOG" | head -1)
rm -f "$LOG"

echo ""
echo "Deployed: $DEPLOY_URL"
echo ""
echo "Aliasing saathi-pink.vercel.app → this deployment..."
npx vercel@latest alias set "$DEPLOY_URL" saathi-pink.vercel.app --scope abhseth-8942s-projects || {
  echo ""
  echo "Alias did not auto-apply. Run manually:"
  echo "  npx vercel@latest alias set $DEPLOY_URL saathi-pink.vercel.app --scope abhseth-8942s-projects"
}

echo ""
echo "Live: https://saathi-pink.vercel.app"
