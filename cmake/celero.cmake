############################
# Celero benchmark library #
############################

set(CELERO_PREFIX ${GLOBAL_OUTPUT_PATH}/celero)

ExternalProject_Add(
  Celero

  UPDATE_COMMAND ""
  PATCH_COMMAND ""

  SOURCE_DIR "${CMAKE_SOURCE_DIR}/ext/Celero-2.0.5"
  CMAKE_ARGS -DCELERO_COMPILE_DYNAMIC_LIBRARIES=OFF -DCMAKE_INSTALL_PREFIX=${CELERO_PREFIX}

  TEST_COMMAND ""
)

set(Celero_LIBRARIES "${CELERO_PREFIX}/lib/${CMAKE_STATIC_LIBRARY_PREFIX}celero${CMAKE_STATIC_LIBRARY_SUFFIX}")
set(Celero_INCLUDE_DIRS "${CMAKE_SOURCE_DIR}/ext/Celero-2.0.5/include/")


include_directories(SYSTEM ${Celero_INCLUDE_DIRS})
