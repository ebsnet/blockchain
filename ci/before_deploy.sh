# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage= \
          full_stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac
    fullstage=$stage/$CRATE_NAME-$TRAVIS_TAG-$TARGET
	mkdir -p $fullstage

    test -f Cargo.lock || cargo generate-lockfile

    # TODO Update this to build the artifacts that matter to you
    ./docker_build_musl.sh

    # TODO Update this to package the right artifacts
    cp target/$TARGET/release/webservice $fullstage/ || cp target/$TARGET/release/webservice.exe $fullstage/
    cp target/$TARGET/release/invoice_generator $fullstage/ || cp target/$TARGET/release/invoice_generator.exe $fullstage/
    cp target/$TARGET/release/tx_generator $fullstage/ || cp target/$TARGET/release/tx_generator.exe $fullstage/

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
