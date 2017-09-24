#pragma once

#include <gtest/gtest.h>

#include <annis/graphstorage/adjacencyliststorage.h>

using namespace annis;

TEST(AdjacencyListStorage, MultiplePathsFindRange)
{

  /*
  +---+
  | 1 | -+
  +---+  |
    |    |
    |    |
    v    |
  +---+  |
  | 2 |  |
  +---+  |
    |    |
    |    |
    v    |
  +---+  |
  | 3 | <+
  +---+
    |
    |
    v
  +---+
  | 4 |
  +---+
    |
    |
    v
  +---+
  | 5 |
  +---+
   */

  AdjacencyListStorage gs;
  gs.addEdge({1, 2});
  gs.addEdge({2, 3});
  gs.addEdge({3, 4});
  gs.addEdge({1, 3});
  gs.addEdge({4, 5});

  std::vector<nodeid_t> found;
  auto it = gs.findConnected(1,3,3);
  for(auto n=it->next(); n; n = it->next())
  {
    found.push_back(*n);
  }

  ASSERT_EQ(2, found.size());

  std::sort(found.begin(),found.end());

  ASSERT_EQ(4, found[0]);
  ASSERT_EQ(5, found[1]);
}

TEST(AdjacencyListStorage, SimpleDAGFindAll)
{
  /*
  +---+     +---+     +---+     +---+
  | 7 | <-- | 5 | <-- | 3 | <-- | 1 |
  +---+     +---+     +---+     +---+
              |         |         |
              |         |         |
              v         |         v
            +---+       |       +---+
            | 6 |       |       | 2 |
            +---+       |       +---+
                        |         |
                        |         |
                        |         v
                        |       +---+
                        +-----> | 4 |
                                +---+
  */
  AdjacencyListStorage gs;
  gs.addEdge({1, 2});
  gs.addEdge({2, 4});
  gs.addEdge({1, 3});
  gs.addEdge({3, 4});
  gs.addEdge({3, 5});
  gs.addEdge({5, 6});
  gs.addEdge({5, 7});

  std::vector<nodeid_t> found;
  auto it = gs.findConnected(1, 1, std::numeric_limits<uint32_t>::max());

  for(auto n=it->next(); n; n = it->next())
  {
    found.push_back(*n);
  }


  // make sure that 4 is only found once
  ASSERT_EQ(6, found.size());

  std::sort(found.begin(),found.end());

  ASSERT_EQ(2, found[0]);
  ASSERT_EQ(3, found[1]);
  ASSERT_EQ(4, found[2]);
  ASSERT_EQ(5, found[3]);
  ASSERT_EQ(6, found[4]);
  ASSERT_EQ(7, found[5]);
}
