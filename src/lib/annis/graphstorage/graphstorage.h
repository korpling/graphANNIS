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

#pragma once

#include <annis/annostorage.h>          // for AnnoStorage
#include <annis/types.h>                // for nodeid_t, GraphStatistic
#include <stdlib.h>                     // for size_t
#include <cereal/types/polymorphic.hpp>  // for base_class
#include <memory>                       // for unique_ptr
#include <vector>                       // for vector
#include <annis/iterators.h>

namespace annis { class DB; }
namespace annis { class StringStorage; }
namespace annis { class EdgeIterator; }


namespace annis
{

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

  virtual const BTreeMultiAnnoStorage<Edge>& getAnnoStorage() const = 0;

  virtual std::shared_ptr<AnnoIt> getSourceNodeIterator() const = 0;

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
