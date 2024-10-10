#pragma once
#include <cstddef>

template<typename T, typename N>
class Function {
    void *state_pointer;

    N (*fn_pointer)(const void *, T, const N *);

    void (*destructor)(void *);

public:
    N operator()(T time, const N *inputs) const {
        return fn_pointer(state_pointer, time, inputs);
    }
};

template<typename T, typename N>
struct CauchyTask {
    std::size_t size;
    const Function<T, N> *derivatives;
    const N *initial_conditions;
    T initial_time;
};

