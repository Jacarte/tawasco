# Compile
# Test using several features
#
# NOTE: medium is enough MacBook pro Intel i9
FEATURES="10000 20000 30000 70000 100000" # large"
EXPECTED="My password"




echo "Testing eviction"

for feat in $FEATURES
do
  rm -rf target
  cargo build --release --no-default-features

  TOTAL=0
  SUCC=0

  for i in $(seq 0 100)
  do
    OUT=$(TRIES=$feat ./target/release/eviction 2> ../plots/samples.py)
    if [ "$OUT" == "$EXPECTED" ]
    then
      ((SUCC=SUCC + 1))
    else
      echo "$OUT" # > /dev/null
    fi

    ((TOTAL=TOTAL + 1))
    # sleep 5

    printf "\rEviction accuracy $SUCC/$TOTAL '$feat'"

  done
done

