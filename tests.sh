for i in `seq 1 1000`; do
    out=$(cargo test 2>&1)
    if [ $? != 0 ]; then
        echo "wah, tests are flaky"
        echo "$out"
    fi
    if [ $(($i % 5)) = 0 ]; then
        echo $i
    fi
done    
