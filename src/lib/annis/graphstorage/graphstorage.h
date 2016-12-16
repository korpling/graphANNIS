#pragma once

#include <stdlib.h>
#include <cstdint>
#include <string>
#include <vector>
#include <memory>

#include <cereal/types/polymorphic.hpp>

#include <annis/types.h>
#include <annis/iterators.h>
#include <annis/stringstorage.h>

#include <annis/annostorage.h>

namespace annis
{
class DB;

class ReadableGraphStorage
{
public:

  ReadableGraphStorage()
  {
    stat.valid = false;
  }

  virtual ~ReadableGraphStorage() {}

  virtual void copy(const DB& db, const ReadableGraphStorage& orig) = 0;

  virtual void clear() = 0;

  virtual bool isConnected(const Edge& edge, unsigned int minDistance = 1, unsigned int maxDistance = 1) const = 0;
  /**
   * @brief Returns a newly allocated iterator for the connected nodes.
   * @param sourceNode
   * @param minDistance
   * @param maxDistance
   * @return An iterator.
   */
  virtual std::unique_ptr<EdgeIterator> findConnected(
                                           nodeid_t sourceNode,
                                           unsigned int minDistance = 1,
                                           unsigned int maxDistance = 1) const = 0;

  virtual int distance(const Edge& edge) const = 0;

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) const = 0;
  virtual std::vector<nodeid_t> getOutgoingEdges(nodeid_t node) const = 0;


  virtual size_t numberOfEdges() const = 0;
  virtual size_t numberOfEdgeAnnotations() const = 0;

  virtual size_t guessMaxAnnoCount(const StringStorage& strings, const Annotation& anno) const = 0;

  virtual GraphStatistic getStatistics() const
  {
    return stat;
  }

  virtual void calculateStatistics(const StringStorage& strings) {}

  virtual size_t estimateMemorySize() = 0;

  template<class Archive>
  void serialize(Archive & archive)
  {
    archive(stat);
  }

protected:
  GraphStatistic stat;
};

class WriteableGraphStorage : public ReadableGraphStorage
{
public:

  virtual ~WriteableGraphStorage() {}

  virtual void addEdge(const Edge& edge) = 0;
  virtual void addEdgeAnnotation(const Edge& edge, const Annotation& anno) = 0;


  virtual void deleteEdge(const Edge& edge) = 0;
  virtual void deleteNode(nodeid_t node) = 0;
  virtual void deleteEdgeAnnotation(const Edge& edge, const AnnotationKey& anno) = 0;


  virtual void calculateIndex() {}

  template<class Archive>
  void serialize(Archive & archive)
  {
    archive(cereal::base_class<ReadableGraphStorage>(this));
  }

};


} // end namespace annis
