let bin = "nur"
let os = ($env.OS | parse "{name}-{version}" | first)
let target = $env.TARGET
let format = $env.FORMAT
let src = $env.GITHUB_WORKSPACE
let version = (open Cargo.toml | get package.version)
let suffix = match [$os.name, $format] {
    ["windows", "msi"] => ".msi"
    ["windows", "bin"] => ".exe"
    _ => ""
}
let release_bin = match [$os.name, $format] {
    ["windows", "msi"] => $'target/release/($bin)($suffix)'
    _ => $'target/($target)/release/($bin)($suffix)'
}
let executables = match [$os.name, $format] {
    ["windows", "msi"] => $'target/wix/($bin)*($suffix)'
    _ => $'target/($target)/release/($bin)*($suffix)'
}
let dist = $'($env.GITHUB_WORKSPACE)/output'
let dest = $'($bin)-($version)-($target)'

print $'Config for this run is:'
print {
    bin: $bin
    os: $os
    target: $target
    format: $format
    src: $src
    version: $version
    suffix: $suffix
    release_bin: $release_bin
    executables: $executables
    dist: $dist
    dest: $dest
}

print $'Packaging ($bin) v($version) for ($target) in ($src)...'

print $'Preparing build dependencies for ($bin)...'
match [$os.name, $target] {
    ["ubuntu", "aarch64-unknown-linux-gnu"] => {
        sudo apt update
        sudo apt install -y gcc-aarch64-linux-gnu
        $env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = 'aarch64-linux-gnu-gcc'
    }
}

print $'Start building ($bin)...'
match [$os.name, $format] {
    ["windows", "msi"] => {
        cargo install cargo-wix
        cargo build --release --all  # wix needs target/release
        cargo wix --no-build --nocapture --package $bin --output $"target/wix/($dest).msi"
    }
    ["windows", "bin"] => {
        cargo build --release --all --target $target
    }
    [_, "bin"] => {
        cargo build --release --all --target $target --features=static-link-openssl
    }
}

print $'Check ($bin) version...'
let built_version = do --ignore-errors { ^$release_bin --version } | str join
if ($built_version | str trim | is-empty) {
    print $'(ansi r)Incompatible arch: cannot run ($release_bin)(ansi reset)'
} else {
    print $" -> built version is: ($built_version)"
}

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
        cp $'($dest)/($dest).msi' $'($dist)/'
        $archive = $'($dist)/($dest).msi'
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
