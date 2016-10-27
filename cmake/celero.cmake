############################
# Celero benchmark library #
############################

ExternalProject_Add(
  Celero

  UPDATE_COMMAND ""
  PATCH_COMMAND ""

  SOURCE_DIR "${CMAKE_SOURCE_DIR}/ext/Celero-2.0.5"
  CMAKE_ARGS -DCELERO_COMPILE_DYNAMIC_LIBRARIES=OFF -DCMAKE_INSTALL_PREFIX=${GLOBAL_OUTPUT_PATH}/celero

  TEST_COMMAND ""
)


set(Celero_LIBRARIES "${CMAKE_STATIC_LIBRARY_PREFIX}celero${CMAKE_STATIC_LIBRARY_SUFFIX}")
set(Celero_INCLUDE_DIRS "${CMAKE_SOURCE_DIR}/ext/Celero-2.0.5/include/")

ExternalProject_Get_Property(Celero BINARY_DIR)

ExternalProject_Add_Step(
  Celero CopyToBin
  COMMAND ${CMAKE_COMMAND} -E copy_directory ${GLOBAL_OUTPUT_PATH}/celero/lib ${GLOBAL_OUTPUT_PATH}
  DEPENDEES install
)



include_directories(SYSTEM ${Celero_INCLUDE_DIRS})
