# Compile
# Test using several features
#
# NOTE: medium is enough MacBook pro Intel i9
FEATURES="small medium large extra" # large"
EXPECTED="My password"




echo "Testing eviction"

for feat in $FEATURES
do
  rm -rf target
  cargo build --release --features "$feat" --no-default-features

  TOTAL=0
  SUCC=0

  for i in $(seq 0 100)
  do
    OUT=$(./target/release/eviction)
    if [ "$OUT" == "$EXPECTED" ]
    then
      ((SUCC=SUCC + 1))
    else
      echo "$OUT"
    fi

    ((TOTAL=TOTAL + 1))
    sleep 5


  done
  echo "Eviction accuracy $SUCC/$TOTAL '$feat'"
done

