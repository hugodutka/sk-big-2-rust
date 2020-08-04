for i in `seq 1 100`; do
    cargo test > /dev/null 2>&1
    if [ $? != 0 ]; then
        echo "wah, tests are flaky"
    fi
    if [ $(($i % 5)) = 0 ]; then
        echo $i
    fi
done    
