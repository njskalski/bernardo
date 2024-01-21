#include <cstdio>
#include <vector>

#include "hello.h"

int main() {
    printf("get some number: %d", get_some_number());

    std::vector<int> some{0, 0, 0, 0, 0};
    fill_array(some);

    return 0;
}