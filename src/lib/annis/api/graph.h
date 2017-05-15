#pragma once

#include <string>
#include <vector>

namespace annis {
namespace api {

/**
 * @brief The Label struct
 */
struct Label
{
  std::string ns;
  std::string name;
  std::string value;
};

/**
 * @brief The Node struct
 */
struct Node
{
  std::string id;
  std::string type;
  std::vector<Label> labels;
};

/**
 * @brief The Edge struct
 */
struct Edge
{
  std::string sourceID;
  std::string targetID;
  std::vector<Label> labels;
};

/**
 * @brief A simple labeled graph implementation.
 */
class Graph
{
public:



public:
  Graph();

  std::vector<Node> nodes;
  std::vector<Edge> edges;
};

}
}

