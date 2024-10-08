cmake_minimum_required(VERSION 3.5)

project(solvers)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_FLAGS "-Wall -Wextra -Wpedantic")

add_library(runge-kutta SHARED src/runge_kutta.cpp include/ffi.h)
target_include_directories(runge-kutta PUBLIC include/)
