#include <solver.h>
#include <vector>

template<typename T, typename N>
class AdamsBashforth : public Solver<T, N> {
    T h;
    T current_time[2];
    std::vector<N> last_solution[2];

public:
    explicit AdamsBashforth(T step = 0.1) {
        this->h = step;
    }

    void prepare_for_task(CauchyTask<T, N> task) override {
        auto cond_view = task.initial_conditions;
        current_time[0] = task.initial_time;
        last_solution[0] = {cond_view, cond_view + task.size};

        // Do one step with Euler method to populate last_solution[1]
        auto view = task.derivatives;
        last_solution[1].resize(task.size);
        for (std::size_t i = 0; i < task.size; ++i) {
            auto &f = view[i];
            last_solution[1][i] = last_solution[0][i] + h * f(current_time[0], last_solution[0].data());
        }
        current_time[1] = current_time[0] + h;
    }

    N *next_solution(CauchyTask<T, N> task, T &out_time) override {
        auto view = task.derivatives;
        auto size = task.size;
        std::vector<N> result;
        result.resize(size);

        for (std::size_t i = 0; i < size; i++) {
            auto &f = view[i];

            // y n = y n-1 + h(1.5 f(x n-1, y n-1) - 0.5 f(x n-2, y n-2))
            result[i] = last_solution[1][i] + h * (3. / 2. * f(current_time[1], last_solution[1].data()) -
                                                   1. / 2. * f(current_time[0], last_solution[0].data()));
        }

        current_time[0] = current_time[1];
        current_time[1] += h;
        out_time = current_time[1];
        // move y n-1 to y n-2
        last_solution[0].swap(last_solution[1]);
        // move result to y n-1
        last_solution[1].swap(result);
        return last_solution[1].data();
    }
};

template<typename T, typename N>
std::unique_ptr<Solver<T, N>> GLOBAL_SOLVER = std::make_unique<AdamsBashforth<T, N>>();