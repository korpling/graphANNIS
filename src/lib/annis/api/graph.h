#pragma once

#include <string>
#include <memory>
#include <annis/db.h>

namespace annis {
namespace api {

class Graph
{
public:
  Graph();

  void addNode(std::string name);

private:
  std::shared_ptr<annis::DB> db;
};

}
}

