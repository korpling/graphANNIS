/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#include "graphupdate.h"


using namespace annis::api;

GraphUpdate::GraphUpdate()
 : lastConsistentChangeID(0)
{
}

void GraphUpdate::addNode(std::string name, std::string type)
{
  std::shared_ptr<AddNodeEvent> evt = std::make_shared<AddNodeEvent>();
  evt->changeID = lastConsistentChangeID + diffs.size() + 1;
  evt->nodeName = name;
  evt->nodeType = type;

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

void GraphUpdate::addEdgeLabel(std::string sourceNode, std::string targetNode, std::string layer,
                               std::string componentType, std::string componentName,
                               std::string annoNs, std::string annoName, std::string annoValue)
{
    std::shared_ptr<AddEdgeLabelEvent> evt = std::make_shared<AddEdgeLabelEvent>();
    evt->changeID = lastConsistentChangeID + diffs.size() + 1;

    evt->sourceNode = sourceNode;
    evt->targetNode = targetNode;
    evt->layer = layer;
    evt->componentType = componentType;
    evt->componentName = componentName;

    evt->annoNs = annoNs;
    evt->annoName = annoName;
    evt->annoValue = annoValue;

    diffs.push_back(evt);

}

void GraphUpdate::deleteEdgeLabel(std::string sourceNode, std::string targetNode, std::string layer,
                                  std::string componentType, std::string componentName,
                                  std::string annoNs, std::string annoName)
{
   std::shared_ptr<DeleteEdgeLabelEvent> evt = std::make_shared<DeleteEdgeLabelEvent>();
   evt->changeID = lastConsistentChangeID + diffs.size() + 1;

   evt->sourceNode = sourceNode;
   evt->targetNode = targetNode;
   evt->layer = layer;
   evt->componentType = componentType;
   evt->componentName = componentName;

   evt->annoNs = annoNs;
   evt->annoName = annoName;

   diffs.push_back(evt);
}

void GraphUpdate::finish()
{
   if(!diffs.empty())
   {
      std::shared_ptr<UpdateEvent> lastEvent = *(diffs.rbegin());
      lastConsistentChangeID = lastEvent->changeID;
   }
}

bool GraphUpdate::isConsistent() const
{
   if(diffs.empty())
   {
      return true;
   }
   else
   {
      std::shared_ptr<UpdateEvent> lastEvent = *(diffs.rbegin());
      if(lastEvent)
      {
        return lastConsistentChangeID == lastEvent->changeID;
      }
      else
      {
        return false;
      }
   }
}
