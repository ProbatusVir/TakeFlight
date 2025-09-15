# Some notes on testing
## You cannot run a batch test command on this project!
This is kinda jank. Because this project is a binary only, we cannot do traditional integration testing, nor can we isolate our testing modules very well.

Consequently, if you want to test one module, and not another, you should go into the file containing those tests and run those. 

### Currently the manual testing modules include
* camera_conversion_test (an integration test)
* stream_tests (The encoding is lossy, so testing that it is actually working correctly is a little difficult)