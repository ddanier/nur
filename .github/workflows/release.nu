let bin = "nur"
let os = $env.OS
let target = $env.TARGET
let format = $env.FORMAT
let src = $env.GITHUB_WORKSPACE
let version = (open Cargo.toml | get package.version)
let suffix = if $os == 'windows-latest' { '.exe' }
let nur_bin = $'target/($target)/release/($bin)($suffix)'
let executables = $'target/($target)/release/($bin)*($suffix)'
let dist = $'($env.GITHUB_WORKSPACE)/output'
let dest = $'($bin)-($version)-($target)'

print $'Packaging ($bin) v($version) for ($target) in ($src)...'

print $'Preparing build dependencies...'
match [$os, $target] {
    ["ubuntu-20.04", "aarch64-unknown-linux-gnu"] => {
        sudo apt update
        sudo apt install -y gcc-aarch64-linux-gnu
        $env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = 'aarch64-linux-gnu-gcc'
    }
}

print $'Start building ($bin)...'
match $format {
    "bin" => {
        cargo build --release --all --target $target
    }
    "msi" => {
        cargo install cargo-wix
        cargo build --release --all  # wix needs target/release
        cargo wix --no-build --nocapture --package $bin --output #TODO
    }
}


print $'Check ($bin) version...'
let ver = do { ^$nur_bin --version } | str join
if ($ver | str trim | is-empty) {
    print $'(ansi r)Incompatible arch: cannot run ($bin)(ansi reset)'
} else {
    print $ver
}

print $'Cleanup release...'
rm -rf ...(glob $'target/($target)/release/*.d')

print $'Copying ($bin) and other release files to ($dest)...'
mkdir $dest
[README.md LICENSE ...(glob $executables)] | each {|it| cp -rv $it $dest } | flatten

print $'Creating release archive...'
mkdir $dist
mut archive = $'($dist)/($dest).tar.gz'
match $os {
    "windows-latest" => {
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
