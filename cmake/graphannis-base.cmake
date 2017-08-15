ExternalProject_Add(
    graphannis-base
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo build COMMAND cargo build --release
    BINARY_DIR "${CMAKE_SOURCE_DIR}/graphannis-base"
    INSTALL_COMMAND ""
    LOG_BUILD ON)

  set(GRAPHANNISBASE_LIBRARIES "${CMAKE_SOURCE_DIR}/graphannis-base/target/release/${CMAKE_STATIC_LIBRARY_PREFIX}graphannis_base${CMAKE_STATIC_LIBRARY_SUFFIX}" "dl")
  set(GRAPHANNISBASE_INCLUDE_DIRS "${CMAKE_SOURCE_DIR}/graphannis-base/include/")
