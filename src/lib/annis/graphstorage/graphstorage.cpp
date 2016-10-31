#include <annis/graphstorage/graphstorage.h>

#include <cereal/archives/binary.hpp>

#include <fstream>

using namespace annis;
using namespace std;


bool ReadableGraphStorage::load(string dirPath)
{
  stat.valid = false;
  ifstream in;

  in.open(dirPath + "/statistics.cereal");
  if(in.is_open())
  {
    cereal::BinaryInputArchive ar(in);
    ar(stat);
  }
  return true;
}

bool ReadableGraphStorage::save(string dirPath)
{
  ofstream out;

  out.open(dirPath + "/statistics.cereal");
  cereal::BinaryOutputArchive ar(out);
  ar(stat);

  return true;
}
