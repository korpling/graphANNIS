#include "graphupdate.h"

#include <annis/types.h>


#include <boost/thread/shared_lock_guard.hpp>


using namespace annis::api;

GraphUpdate::GraphUpdate()
{
}

void GraphUpdate::addNode(std::string name)
{
  std::shared_ptr<AddNodeEvent> evt = std::make_shared<AddNodeEvent>();
  evt->changeID = lastConsistentChangeID + diffs.size() + 1;
  evt->nodeName = name;

  diffs.push_back(evt);
}

void GraphUpdate::deleteNode(std::string name)
{
  std::shared_ptr<DeleteNodeEvent> evt = std::make_shared<DeleteNodeEvent>();
  evt->changeID = lastConsistentChangeID + diffs.size() + 1;
  evt->nodeName = name;

  diffs.push_back(evt);
}

void GraphUpdate::addNodeLabel(std::string nodeName, std::string ns, std::string name, std::string value)
{
  std::shared_ptr<AddNodeLabelEvent> evt = std::make_shared<AddNodeLabelEvent>();
  evt->changeID = lastConsistentChangeID + diffs.size() + 1;
  evt->nodeName = nodeName;
  evt->annoNs = ns;
  evt->annoName = name;
  evt->annoValue = value;

  diffs.push_back(evt);
}

void GraphUpdate::deleteNodeLabel(std::string nodeName, std::string ns, std::string name)
{
   std::shared_ptr<DeleteNodeLabelEvent> evt = std::make_shared<DeleteNodeLabelEvent>();
   evt->changeID = lastConsistentChangeID + diffs.size() + 1;
   evt->nodeName = nodeName;
   evt->annoNs = ns;
   evt->annoName = name;

   diffs.push_back(evt);
}

void GraphUpdate::addEdge(std::string sourceNode, std::string targetNode, std::string layer,
                          std::string componentType, std::string componentName)
{
   std::shared_ptr<AddEdgeEvent> evt = std::make_shared<AddEdgeEvent>();
   evt->changeID = lastConsistentChangeID + diffs.size() + 1;
   evt->sourceNode = sourceNode;
   evt->targetNode = targetNode;
   evt->layer = layer;
   evt->componentType = componentType;
   evt->componentName = componentName;

   diffs.push_back(evt);
}

void GraphUpdate::deleteEdge(std::string sourceNode, std::string targetNode, std::string layer,
                             std::string componentType, std::string componentName)
{
   std::shared_ptr<DeleteEdgeEvent> evt = std::make_shared<DeleteEdgeEvent>();
   evt->changeID = lastConsistentChangeID + diffs.size() + 1;
   evt->sourceNode = sourceNode;
   evt->targetNode = targetNode;
   evt->layer = layer;
   evt->componentType = componentType;
   evt->componentName = componentName;

   diffs.push_back(evt);
}

void GraphUpdate::finish()
{
   if(!diffs.empty())
   {
      std::shared_ptr<UpdateEvent> evt = *(diffs.rbegin());
      lastConsistentChangeID = evt->changeID;
   }
}
