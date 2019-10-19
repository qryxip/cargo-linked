exit-with-usage() {
  echo "Usage: $0 <target>" 1>&2
  exit 1
}

while getopts ":h:" OPT; do
  case "$OPT" in
    \?) exit-with-usage;;
  esac
done

if [ "$OPTIND" != "$#" ]; then
  exit-with-usage
fi

shift "$(($OPTIND - 1))"
target="$1"

if [[ "$target" =~ pc-windows ]]; then
  EXE=.exe
fi
EXECUTABLE="./target/release/${GITHUB_REPOSITORY#*/}$EXE"
ASSET_STEM="${GITHUB_REPOSITORY#*/}-${GITHUB_REF#refs/tags/}-$target"
git archive -o "./$ASSET_STEM.tar" --prefix "$ASSET_STEM/" HEAD
tar -xf "./$ASSET_STEM.tar"
mv "$EXECUTABLE" "./$ASSET_STEM/"
if [[ "$target" =~ pc-windows ]]; then
  ASSET="./$ASSET_STEM.zip"
  7z a "$ASSET" "./$ASSET_STEM"
  zipinfo "$ASSET"
else
  ASSET="./$ASSET_STEM.tar.gz"
  tar -czvf "$ASSET" "./$ASSET_STEM"
fi
echo "::set-output name=asset::$ASSET"
