#include "hello.hpp"

int get_some_number() {
    return 4;
}

void fill_array(std::vector<int> &array) {
    int counter = 0;

    for (std::size_t i = 0; i < array.size(); ++i) {
        array[i] = counter;
        counter++;
    }
}
