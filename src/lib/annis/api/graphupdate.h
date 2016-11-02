#pragma once

#include <string>
#include <memory>
#include <annis/db.h>

#include <list>
#include <string>

#include <cereal/types/list.hpp>

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
  void addLabel(std::string nodeName, std::string ns, std::string name, std::string value);

  /**
   * @brief Delete an existing label from a node.
   *
   * If the node or the label does not exist, nothing is done.
   *
   * @param nodeName
   * @param ns
   * @param name
   */
  void deleteLabel(std::string nodeName, std::string ns, std::string name);

private:

  friend class CorpusStorage;

  enum Type
  {
    add_node, delete_node, add_label, delete_label
  };

  struct Event
  {
    Type type;
    std::string arg0;
    std::string arg1;
    std::string arg2;
    std::string arg3;
  };

  template<class Archive>
  void serialize(Archive & archive,
                 Event & evt)
  {
    archive(evt.type, evt.arg0, evt.arg1, evt.arg2, evt.arg3);
  }

  template<class Archive>
  void serialize(Archive & archive)
  {
    archive(diffs);
  }


private:
  std::list<Event> diffs;
};

}
}

