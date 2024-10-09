#pragma once
#include <cstddef>
#include <span>

template<typename T, typename N>
class Function {
    void *state_pointer = nullptr;
    N (*fn_pointer)(const void *, T, const N *) = nullptr;
    void *(*destructor)(void *) = nullptr;

public:
    N operator()(T time, const N* inputs) const {
        return fn_pointer(state_pointer, time, inputs);
    }

    Function(const Function &other) = delete;
    Function(Function &&other) noexcept = delete;
    Function & operator=(const Function &other) = delete;
    Function & operator=(Function &&other) noexcept = delete;
};

template<typename T, typename N>
class CauchyTask {
    std::size_t size = 0;
    T initial_time;
    Function<T, N> *derivatives = nullptr;
    N *initial_conditions = nullptr;

public:
    [[nodiscard]] T get_initial_time() const {
        return initial_time;
    }

    [[nodiscard]] std::span<N> get_initial_conditions() const {
        return {initial_conditions, size};
    }

    [[nodiscard]] std::span<Function<T, N>> get_functions() const {
        return {derivatives, size};
    }
};
