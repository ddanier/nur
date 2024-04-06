let bin = "nur"
let os = ($env.OS | parse "{name}-{version}" | first)
let target = $env.TARGET
let format = $env.FORMAT
let src = $env.GITHUB_WORKSPACE
let version = (open Cargo.toml | get package.version)
let suffix = match $os.name {
    "windows" => ".exe"
    _ => ""
}
let target_path = match [$os.name, $format] {
    ["windows", "msi"] => $'target/release/'
    _ => $'target/($target)/release/'
}
let release_bin = $'($target_path)/($bin)($suffix)'
let executables = $'($target_path)/($bin)*($suffix)'
let dist = $'($env.GITHUB_WORKSPACE)/output'
let dest = $'($bin)-($version)-($target)'

print $'Config for this run is:'
{
    bin: $bin
    os: $os
    target: $target
    format: $format
    src: $src
    version: $version
    suffix: $suffix
    target_path: $target_path
    release_bin: $release_bin
    executables: $executables
    dist: $dist
    dest: $dest
} | table -e

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

print $'Cleanup release target path...'
rm -rf ...(glob $'($target_path)/*.d')

print $'Copying ($bin) and other release files to ($dest)...'
match [$os.name, $format] {
    ["windows", "msi"] => {
        print ' -> skipping for MSI build'
    }
    _ => {
        mkdir $dest
        [README.md LICENSE ...(glob $executables)] | each {|it| cp -rv $it $dest } | flatten
    }
}

print $'Creating release archive in ($dist)...'
mkdir $dist
mut archive = $'($dist)/($dest).tar.gz'
match [$os.name, $format] {
    ["windows", "msi"] => {
        $archive = $'($dist)/($dest).msi'
        cargo wix --no-build --nocapture --package $bin --output $archive
    }
    ["windows", "bin"] => {
        $archive = $'($dist)/($dest).zip'
        7z a $archive $dest
    }
    _ => {
        tar -czf $archive $dest
    }
}
print $' -> archive: ($archive)'

print $'Provide archive to GitHub...'
echo $"archive=($archive)" | save --append $env.GITHUB_OUTPUT
