# Phobia
Phobia is a asynchronous multi-threaded scenario-based load-generator implemented in Rust for stress testing services.

# Usage
    phobia <path> --concurency <concurency> --scale <scale> --step <step>
    -h, --help       Prints help information
    -V, --version    Prints version information
    -c, --concurency <concurency>    
        --scale <scale>              
    -s, --step <step>          
    
# Sample Scenario
```yaml
- host: "<HOST>"
  start: 10
  end: 90
  path: "/"
  method: POST
  content-type: multipart
  body:
    path: "<Path to File>"
    name: audio
- host: "<HOST>"
  start: 23
  end: 42
  path: "/"
  method: POST
  content-type: multipart
  body:
    path: "<Path to File>"
    name: audio
- host: "<HOST>"
  start: 45
  end: 70
  path: "/"
  method: POST
  content-type: multipart
  body:
    path: "<Path to File>"
    name: audio
- host: "<HOST>"
  start: 45
  end: 84
  path: "/"
  method: POST
  content-type: multipart
  body:
    path: "<Path to File>"
    name: audio

```
