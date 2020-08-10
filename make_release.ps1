Get-ChildItem "Cargo.toml" | ForEach-Object {
	$conf = $_ | Get-Content -raw
	$conf -match 'version\s+=\s+"(.*)"' | out-null
	$script:LEVEL99_VERSION = $matches[1]
}

git tag -d $LEVEL99_VERSION
git push --delete origin $LEVEL99_VERSION
git tag $LEVEL99_VERSION
git push --tags
