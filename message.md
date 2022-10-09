<!-- Thank you for filing this! Please read the [debugging tips](https://github.com/gfx-rs/wgpu/wiki/Debugging-wgpu-Applications).
That may let you investigate on your own, or provide additional information that helps us to assist.-->

**Description**
Creating a storage buffer array `using create_buffer_init()`, and placing it in a bind group with layout entry `ty: BindingType::Buffer { ty: BufferBindingType::Storage, ..}` and count `N > 0`, triggers the vulkan validation error, when bound in a render_pass and run using `draw_indexed()`

**Repro steps**
Ideally, a runnable example we can check out.

**Expected vs observed behavior**
There should probably be no validation error triggered

**Extra materials**
`[2022-10-08T10:21:08Z ERROR wgpu_hal::vulkan::instance] 	objects: (type: DESCRIPTOR_SET, hndl: 0xa2eb680000000026, name: Materials Bind Group)`
`[2022-10-08T10:21:08Z ERROR wgpu_hal::vulkan::instance] VALIDATION [VUID-vkCmdDrawIndexed-None-02699 (0xa44449d4)]
    	Validation Error: [ VUID-vkCmdDrawIndexed-None-02699 ] Object 0: handle = 0xa2eb680000000026, name = Materials Bind Group, type = VK_OBJECT_TYPE_DESCRIPTOR_SET; | MessageID = 0xa44449d4 | Descriptor set VkDescriptorSet 0xa2eb680000000026[Materials Bind Group] encountered the following validation error at vkCmdDrawIndexed time: Descriptor in binding #1 index 1 is being used in draw but has never been updated via vkUpdateDescriptorSets() or a similar call. The Vulkan spec states: Descriptors in each bound descriptor set, specified via vkCmdBindDescriptorSets, must be valid if they are statically used by the VkPipeline bound to the pipeline bind point used by this command (https://www.khronos.org/registry/vulkan/specs/1.3-extensions/html/vkspec.html#VUID-vkCmdDrawIndexed-None-02699)
`

**Platform**
Linux kernel 5.19.12, running on a Ryzen7 3700U, using wgpu version 0.14

