#include "graphupdate.h"

#include <boost/thread/shared_lock_guard.hpp>

using namespace annis::api;

GraphUpdate::GraphUpdate()
{
}

void GraphUpdate::addNode(std::string name)
{
  diffs.push_back({lastConsistentChangeID + diffs.size() + 1, add_node, name, "", "", ""});
}

void GraphUpdate::deleteNode(std::string name)
{
  diffs.push_back({lastConsistentChangeID + diffs.size() + 1, delete_node, name, "", "", ""});
}

void GraphUpdate::addNodeLabel(std::string nodeName, std::string ns, std::string name, std::string value)
{
  diffs.push_back({lastConsistentChangeID + diffs.size() + 1, add_node_label, nodeName, ns, name, value});
}

void GraphUpdate::deleteNodeLabel(std::string nodeName, std::string ns, std::string name)
{
   diffs.push_back({lastConsistentChangeID + diffs.size() + 1, delete_node_label, nodeName, ns, name, ""});
}

void GraphUpdate::finish()
{
   if(!diffs.empty())
   {
      lastConsistentChangeID = diffs.rbegin()->changeID;
   }
}
