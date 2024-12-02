Every once in a while, when launching the kernel in QEMU, even the bootloader wouldn't boot. It seemed that UEFI did not
detect the ESP at all. I did not understand how this could be, as my image building script definitely did everything properly,
creating the partition with the correct name and OSType and moving all the correct files into the correct places.
    I started to investigate this, creating a loop device to represent the ESP and mounting that into the filesystem
hierarchy. I then listed all the contents of the ESP with the standard 'ls'-utility, and saw all the files in their correct
places. I could even view the sizes of the files, and saw that at least running out of space in the ESP wasn't the problem,
which I had suspected might be the case. So far so good.
    But then something weird happened when I tried to show the contents of limine.cfg: 'cat' failed to do so, and an
input/output error was reported. After some searching online I discovered that the 'dmesg'-utility might show a more detailed
cause for the error. I tried it, and sure enough, there it was, the reason for the error: an attempt to access beyond the
end of the partition. But how could that be, there was still plenty of space left in the partition!
    After some experimentation with attempting to access the various files and viewing dmesg afterwards I narrowed down
that yes, indeed, it was the file accesses that were prompting the accesses beyond the partition's bounds. Assuming that
this wasn't a bug, I could see only one cause for this: the files had to be at least partly located outside the partition's
boundaries. Because of my knowledge of the fundamentals of data storage in computers, and especially of Linux filesystems,
I knew that files were stored as collections of contiguous data blocks on disk. Even though the sizes of those data blocks
in total might not be too large to fit inside the partition, they could be positioned outside of it.
    I thought that there must be an utility that can list all the data blocks of a file, and after some searching online,
I found out about one called 'filefrag'. Using it I discovered that my suspection had been true: the files, althought not too
large, were nevertheless located outside the partition (in one contiguous data block). I wondered whether this might be due
to mcopy (the command I was using to copy the necessary files over into the image file's ESP) first placing the copied files
into the destination, and only afterwards deleting the previous versions of those files, in the case of name clashes. This
might cause mcopy to place the new files into ever-increasing physical addresses, as the files would be placed into physical
addresses just above those of the files currently residing in the ESP.
    I don't understand why the files wouldn't be placed into the lowest-possible physical addresses, as there would be space
left there after the files had been replaced once, as in the first time they were replaced the files would have to be placed
into physical addresses starting from those of the files previously in the ESP, meaning space would be freed at the lowest
physical addresses. This would seem the logical way to behave, but FAT32 and mcopy are both old enough that this sort of weird
behavior is to be expected. And of course, it might also just be that there is some consideration related to how the actual
physical disks FAT32 was designed for behaved, like it being faster to go forwards in the disk than backwards or for going
backwards to not even be possible except by going so far forward as to end up behind, in the case of a circular storage medium.
    To test my hypothesis, I backed up the old disk_image.bin file, and created a fresh one in its place. I looked at the
positions of the files' data blocks (which filefrag calls extents, a name also used by the ext4-filesystem (and maybe earlier
versions too)). The first file copied, the kernel executable, was positioned at physical offset 2056 (in sectors of 512 bytes),
and the files copied afterwards followed it in subsequent positions. All the files consisted of only one extent. 
    As in almost every case each file only has one extent, I will from here on forth, for simplicity's sake, refer simply to
the position of the file, and not that of its individual extents, unless explicitly stated otherwise.
    I ran the image creation script again, and... nothing happened! The files were still located in the exact same positions.
I created a script to create the image and inspect each of the files' positions in a loop, and after running that for hundreds
of times, there was still no change!
    I wondered whether the outcome might be different if I modified the kernel source code, rebuilt the kernel again,
and then created an image file with the new kernel, i.e. if the change in the files' positions would occur only when copying
a different kernel file over to the ESP. I first tried this manually, and it worked. The new files were positioned right
after where the old files had been. I did it again, with the same result. I created a script to automate this procedure,
inserting a 'println!("A");' statement into the end of the kernel's _start() function in each cycle of the loop using 'sed',
and ran it several hundreds of times.
    The pattern held true. Each time new files were copied, the new first-copied-file (kernel executable) was placed right
after the end of the previous last-copied-file (limine.cfg), followed by all the later-copied files, each placed in a subsequent
physical address or one a few sectors apart, compared to the previous file. But another pattern surfaced too: after reaching
a physical address of 260095 (in sectors), the kernel executable file (whose position was the only one I was examining in
this script) would be split into two extents: the first one ending at that address, and the second one starting at 2056 and
continuing onwards from there. In the next cycle of the script, the file would be unified again into only one extent, and
remain unified ever since, until reaching a physical address of 260095 again. During this time, the ESP would be completely
valid, and the kernel would boot successfully.
    This is quite interesting and significant, as 260095 is an address exactly 2048 sectors far from the end of the image file,
were the address to be considered relative to the start of the image file. This is because 128 * 1024^2 (=1MiB) / 512 = 262144
is the size in sectors of the file, meaning that the largest valid address is 262144 - 1 = 262143, and 262143 - 260095 = 2048.
This would suggest that perhaps the physical address isn't relative to the start of the image file after all, but instead to
the start of the partition, as then this address would be exactly the address of the last sector in the image file.
As mcopy is explicitly instructed to copy the files into the image file at a specific offset, it seems plausible that it is not
aware of the image file at large, but only the portion beginning at the specified offset, i.e. in this case the start of the
partition. It would furthermore be plausible, that mcopy is not aware of the size of the partition either, but only the size
of the image file, as I doubt FAT32 includes any partition metadata in the space allocated for the partition itself. It would
seem to make sense to instead include all partition metadata in the partition table, located well before the first partition in
the file. The unawareness of mcopy over anything beyond the size of the image file and the start of the partition would explain
why it doesn't allocate space for the files beyond the image file's bounds, but does allocate space for them beyond the
partition's bounds.

That's that pattern understood, but why are the files only placed into increasing physical addresses if the kernel file has
changed? I initially thought that this might only happen when the kernel file grows in size, and that it would be because
in that case, the new space for the kernel file can not be allocated by just enlarging the current extent, as the BOOTx64.EFI
and limine.cfg files are already located in physical addresses immediately coming after the kernel's end. As such, the kernel
file would have to be placed, at earliest, if only going forward in physical address space, and not backwards, right after
the limine.cfg file's end (as that file is always the one copied last, and thus placed at the highest physical address).
But this turned out to be entirely wrong. The same phenomenon occurs when the file decreases in size, which I tested, again
hundreds of times, with the help of the previous script modified such that the sed-command deletes such a println!() function
call instead of creating a new one. This would suggest that mcopy just simply checks whether an actually different file is being
copied over, and if so, copies it over, into an increasing physical address, and only deletes the old copy afterwards. If the
same file were to be attempted to copy over, then mcopy simply does nothing to save time.
    But there is still one peculiarity left to explore. Why is the kernel executable file always placed at physical offset
2056, relative to the start of the partition? Earlier I wrote that I assume a FAT32 partition to not store any metatadata
about the partition within the partition itself, but now after searching online I've found that to not be true at all. There
are a lot of FAT32 structures located at the start of the partition, and apparently they end up taking up the first 2056
sectors of the partition.
