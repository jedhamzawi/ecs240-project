# ideas

Directory to store notes, info, and brainstorming for ideas for the project.

This directory may be removed once we decide on a direction.


## static analysis tool for CUDA memory bank issue.

Whenever we invoke kernels in CUDA to do a computation, in a single GPU cycle, multiple threads in a warp(The basic unit of the GPU which consists of 32 threads usually) loads data from different memory addresses so that it perform the operation on the given data, and it completes single operation instruction of the threads in the warp at a single cycle. When accesssing the data(In case of accessing shared memory - it can access registers, global memory and shared memory), it goes to shared memory of the GPU which is divided into 32 banks, it is done in this manner so that parallel memory loads can happen, given that there is no bank conflict(We can think of this as a hash table where we have 32 heads and we fetch some value from inside each of the heads). If we have bank conflict(two different values gives the same hash key), then they have to wait so that all the warps load the data correctly.

We can make a static analysis tool which analyses the CUDA code and predicts that we would have any bank conflicts or not.