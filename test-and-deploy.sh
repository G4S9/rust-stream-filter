#!/usr/bin/env bash

set -euo pipefail

pipenv install
( cd src/api_lambda; cargo test ; cargo lambda build --release )
( cd src/object_lambda; cargo test ; cargo lambda build --release )
pipenv run pytest
pipenv run cdktf deploy
