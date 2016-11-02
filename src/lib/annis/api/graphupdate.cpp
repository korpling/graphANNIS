#include "graphupdate.h"

#include <boost/thread/shared_lock_guard.hpp>

using namespace annis::api;

GraphUpdate::GraphUpdate()
{
}

void GraphUpdate::addNode(std::string name)
{
  diffs.push_back({add_node, name, "", "", ""});
}

void GraphUpdate::deleteNode(std::string name)
{
  diffs.push_back({delete_node, name, "", "", ""});
}

void GraphUpdate::addLabel(std::string nodeName, std::string ns, std::string name, std::string value)
{
  diffs.push_back({add_label, nodeName, ns, name, value});
}

void GraphUpdate::deleteLabel(std::string nodeName, std::string ns, std::string name)
{
  diffs.push_back({delete_label, nodeName, ns, name, ""});
}
