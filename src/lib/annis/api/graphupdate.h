#pragma once

#include <string>
#include <memory>
#include <annis/db.h>

namespace annis {
namespace api {

/**
 * @brief Lists updated that can be performed on a graph.
 *
 * This class is intended to make atomical updates to a graph (as represented by
 * the \class DB class possible.
 */
class GraphUpdate
{
public:
  GraphUpdate();

  void addNode(std::string name);

private:

};

}
}

