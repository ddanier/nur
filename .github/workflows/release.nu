let bin = "nur"
let os = ($env.OS | parse "{name}-{version}")
let target = $env.TARGET
let format = $env.FORMAT
let src = $env.GITHUB_WORKSPACE
let version = (open Cargo.toml | get package.version)
let suffix = if $os.name == 'windows' { '.exe' }
let release_bin = $'target/($target)/release/($bin)($suffix)'
let executables = $'target/($target)/release/($bin)*($suffix)'
let dist = $'($env.GITHUB_WORKSPACE)/output'
let dest = $'($bin)-($version)-($target)'

print $'Packaging ($bin) v($version) for ($target) in ($src)...'

print $'Preparing build dependencies for ($bin)...'
match [$os.name, $target] {
    ["ubuntu", "aarch64-unknown-linux-gnu"] => {
        sudo apt update
        sudo apt install -y gcc-aarch64-linux-gnu
    }
}

print $'Start building ($bin)...'
match [$os.name, $format] {
    ["windows", "msi"] => {
        cargo install cargo-wix
        cargo build --release --all  # wix needs target/release
        cargo wix --no-build --nocapture --package $bin --output #TODO
    }
    [_, "bin"] => {
        cargo build --release --all --target $target
    }
}


# print $'Check ($bin) version...'
# let ver = do { ^$release_bin --version } | str join
# if ($ver | str trim | is-empty) {
#     print $'(ansi r)Incompatible arch: cannot run ($bin)(ansi reset)'
# } else {
#     print $ver
# }

print $'Cleanup release...'
rm -rf ...(glob $'target/($target)/release/*.d')

print $'Copying ($bin) and other release files to ($dest)...'
mkdir $dest
[README.md LICENSE ...(glob $executables)] | each {|it| cp -rv $it $dest } | flatten

print $'Creating release archive in ($dist)...'
mkdir $dist
mut archive = $'($dist)/($dest).tar.gz'
match [$os.name, $format] {
    ["windows", "msi"] => {
        print "NOT SUPPORTED YET"  # TODO
    }
    ["windows", "bin"] => {
        $archive = $'($dist)/($dest).zip'
        7z a $archive $dest
    }
    _ => {
        tar -czf $archive $dest
    }
}

print $'Provide archive to GitHub...'
print $' -> archive: ($archive)'
ls $archive
echo $"archive=($archive)" | save --append $env.GITHUB_OUTPUT
