#pragma once

#include <string>
#include <memory>

#include <vector>
#include <string>

#include <cereal/types/string.hpp>
#include <cereal/types/vector.hpp>
#include <cereal/types/polymorphic.hpp>

namespace annis { namespace api {

enum UpdateEventType
{
  add_node, delete_node, add_node_label, delete_node_label,
  add_edge
};

struct UpdateEvent
{
  std::uint64_t changeID;
  // make this class polymorphic
  virtual ~UpdateEvent() {};

  template<class Archive>
  void serialize( Archive & ar )
  {
     ar( changeID);
  }

};


struct AddNodeEvent : UpdateEvent
{
   std::string nodeName;

   template<class Archive>
   void serialize( Archive & ar )
   {
      ar(cereal::base_class<UpdateEvent>(this), nodeName);
   }
};

struct DeleteNodeEvent : UpdateEvent
{
   std::string nodeName;

   template<class Archive>
   void serialize( Archive & ar )
   {
      ar(cereal::base_class<UpdateEvent>(this), nodeName);
   }
};

struct AddNodeLabelEvent : UpdateEvent
{
   std::string nodeName;
   std::string annoNs;
   std::string annoName;
   std::string annoValue;

   template<class Archive>
   void serialize( Archive & ar )
   {
      ar(cereal::base_class<UpdateEvent>(this), nodeName, annoNs, annoName, annoValue);
   }
};

struct DeleteNodeLabelEvent : UpdateEvent
{
   std::string nodeName;
   std::string annoNs;
   std::string annoName;

   template<class Archive>
   void serialize( Archive & ar )
   {
      ar(cereal::base_class<UpdateEvent>(this), nodeName, annoNs, annoName);
   }
};

struct AddEdgeEvent : UpdateEvent
{
   std::string sourceNode;
   std::string targetNode;
   std::string layer;
   std::string componentType;
   std::string componentName;

   template<class Archive>
   void serialize( Archive & ar )
   {
      ar(cereal::base_class<UpdateEvent>(this), sourceNode, targetNode, layer, componentType, componentName);
   }
};

struct DeleteEdgeEvent : UpdateEvent
{
   std::string sourceNode;
   std::string targetNode;
   std::string layer;
   std::string componentType;
   std::string componentName;

   template<class Archive>
   void serialize( Archive & ar )
   {
      ar(cereal::base_class<UpdateEvent>(this), sourceNode, targetNode, layer, componentType, componentName);
   }
};


/**
 * @brief Lists updated that can be performed on a graph.
 *
 * This class is intended to make atomical updates to a graph (as represented by
 * the \class DB class possible.
 */
class GraphUpdate
{
public:



public:
  GraphUpdate();

  /**
   * @brief Adds an empty node with the given name to the graph.
   * If an node with this name already exists, nothing is done.
   *
   * @param name
   */
  void addNode(std::string name);

  /**
   * @brief Delete a node with the give name from the graph.
   *
   * This will delete all node labels as well. If this node does not exist, nothing is done.
   * @param name
   */
  void deleteNode(std::string name);

  /**
   * @brief Adds a label to an existing node.
   *
   * If the node does not exists or there is already a label with the same namespace and name, nothing is done.
   *
   * @param nodeName
   * @param ns The namespace of the label
   * @param name
   * @param value
   */
  void addNodeLabel(std::string nodeName, std::string ns, std::string name, std::string value);

  /**
   * @brief Delete an existing label from a node.
   *
   * If the node or the label does not exist, nothing is done.
   *
   * @param nodeName
   * @param ns
   * @param name
   */
  void deleteNodeLabel(std::string nodeName, std::string ns, std::string name);

  void addEdge(std::string sourceNode, std::string targetNode,
               std::string layer,
               std::string componentType, std::string componentName);

  void deleteEdge(std::string sourceNode, std::string targetNode,
               std::string layer,
               std::string componentType, std::string componentName);

  /**
   * @brief Mark the current state as consistent.
   */
  void finish();

  template<class Archive>
  void serialize(Archive & archive)
  {
    archive(diffs, lastConsistentChangeID);
  }

  const std::vector<std::shared_ptr<UpdateEvent>>& getDiffs() const
  {
     return diffs;
  }

  std::uint64_t getLastConsistentChangeID() const
  {
     return lastConsistentChangeID;
  }

  bool isConsistent() const;

private:
  std::vector<std::shared_ptr<UpdateEvent>> diffs;

  std::uint64_t lastConsistentChangeID;
};

}
}

#include <cereal/archives/binary.hpp>
#include <cereal/archives/xml.hpp>
#include <cereal/archives/json.hpp>

CEREAL_REGISTER_TYPE(annis::api::AddNodeEvent);
CEREAL_REGISTER_TYPE(annis::api::DeleteNodeEvent);
CEREAL_REGISTER_TYPE(annis::api::AddNodeLabelEvent);
CEREAL_REGISTER_TYPE(annis::api::DeleteNodeLabelEvent);
CEREAL_REGISTER_TYPE(annis::api::AddEdgeEvent);
CEREAL_REGISTER_TYPE(annis::api::DeleteEdgeEvent);
