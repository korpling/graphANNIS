#ifndef ANNISDB_H
#define ANNISDB_H

#include <string>
#include <stx/btree_map>
#include <cstdint>
#include <iostream>

#include "tupel.h"

namespace annis
{

class DB
{
public:
  DB();

  bool loadNodeStorage(std::string file);

  Node getNodeByID(std::uint32_t id);

private:
  stx::btree_map<std::uint32_t, Node> nodes;
  stx::btree_map<std::uint32_t, NodeAnnotation> nodeAnnotations;

  std::vector<std::string> nextCSV(std::istream &in);
};

} // end namespace annis
#endif // ANNISDB_H
