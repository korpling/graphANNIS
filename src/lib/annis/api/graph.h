#pragma once

#include <string>
#include <vector>
#include <map>

namespace annis {
namespace api {


/**
 * @brief The Edge struct
 */
struct Edge
{
  std::uint32_t sourceID;
  std::uint32_t targetID;
  /** Maps a fully qualified label name (seperated by "::") to a label value */
  std::map<std::string, std::string> labels;
};


/**
 * @brief The Node struct
 */
struct Node
{
  std::uint32_t id;
  /** Maps a fully qualified label name (seperated by "::") to a label value */
  std::map<std::string, std::string> labels;

  std::vector<Edge> outgoingEdges;
};

}
}

