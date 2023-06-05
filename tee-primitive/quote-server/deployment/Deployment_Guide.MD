# 1. Content
The TDX quote service is a daemonset designated for nodes with label:

`intel.feature.node.kubernetes.io/tdx: "true"`

and have the right to access `/dev/tdx-guest` in the container/POD runnnrunning ig in either bare metal or VM.

with above requirements, following components are required to be installed or configured:

- NFD(node feature discovery) enabled in Kubernetes Cluster. Note: NFD should be patched to support TDVM labeling
- Intel device plugin operator and TDX Device Plugin daemonset should be enabled in Kubernetes Cluster. Note: operator should be patched and a brand new DX Device Plugin should be implemented.
- udev should be set in Kubernetes Cluster nodes to allow certain user or groups to access /dev/tdx-guest device in container/POD
- quote service daemonset and service should be deployed
- a simple workload to get TDX quote from the quote service 

## 2. install NFD

### get source code
```
git clone https://github.com/intel-sandbox/node-feature-discovery-ccnp.git  
git checkout tdx-nfd-enable
```

### build and import container image
```
cd node-feature-discovery-ccnp/
make image
ctr -n k8s.io image import node-feature-discovery-IMAGE-NAME.tar
```

### deploy NFD
```
cd node-feature-discovery-ccnp/deployment
kubectl apply -k overlays/default
```

### check deploy result
```
root@ubuntu:~# kubectl get pod -n node-feature-discovery
NAME                          READY   STATUS    RESTARTS      AGE
nfd-master-775dcdb4c9-gl4kn   1/1     Running   0             37h
nfd-worker-52m4f              1/1     Running   0             37h
```

## 3. install Intel device plugin operator and per node TDX device plugin server(daemonset)

### get source code
```
git clone https://github.com/intel-sandbox/intel-device-plugins-for-kubernetes-ccnp.git
git checkout tdx-device-plugin-enable
```

### build and import container image
```
cd intel-device-plugins-for-kubernetes-ccnp
docker build -t deviceplugin-operator-tdx:v1   -f build/docker/intel-deviceplugin-operator.Dockerfile .
docker save -o deviceplugin-operator-tdx.tar deviceplugin-operator-tdx:v1
ctr -n k8s.io image import deviceplugin-operator-tdx.tar
```

### deploy NFD labeling rules for TDX
```
cd intel-device-plugins-for-kubernetes-ccnp
kubectl apply -f deployments/nfd/overlays/node-feature-rules/node-feature-rules.yaml
``` 
after the NDF rules are deployed, we can see the node with TDX enabled is labeled as bellow:
```
kubectl get node -o json | jq .items[].metadata.labels | grep tdx
  "feature.node.kubernetes.io/cpu-security.tdx.enabled": "true",
  "intel.feature.node.kubernetes.io/tdx": "true",
```
this label can be used as node selector by TDX device plugin and quote service


### deploy Intel device plugin operator and TDX device plugin daemonset
deplpoy dependent cert-manager first:
```
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.11.0/cert-manager.yaml
```

deploy operator:
```
cd intel-device-plugins-for-kubernetes-ccnp/deployments/
kubectl apply -k  operator/default/
```

deploy TDX device plugin CRD
```
cd intel-device-plugins-for-kubernetes-ccnp/
kubectl apply -f deployments/operator/crd/bases/deviceplugin.intel.com_tdxdeviceplugins.yaml
```

deploy TDX device plugin daemonset
```
cd intel-device-plugins-for-kubernetes-ccnp/
kubectl apply -f deployments/tdx_plugin/base/intel-tdx-plugin.yaml
```

### check deploy result
the PODs
```
root@ubuntu:~#  kubectl get pod -A
NAMESPACE                   NAME                                                     READY   STATUS    RESTARTS         AGE
...
inteldeviceplugins-system   inteldeviceplugins-controller-manager-754b7b67b6-cgnlf   2/2     Running   0                37h
...
kube-system                 intel-tdx-plugin-njfff                                   1/1     Running   0                37h
...
```

the node
```
kubectl describe node 
...
Capacity:
  cpu:                      8
  ephemeral-storage:        73181444Ki
  hugepages-1Gi:            0
  hugepages-2Mi:            0
  memory:                   7687708Ki
  pods:                     110
  tdx.intel.com/tdx-guest:  110
Allocatable:
  cpu:                      8
  ephemeral-storage:        67444018679
  hugepages-1Gi:            0
  hugepages-2Mi:            0
  memory:                   7585308Ki
  pods:                     110
  tdx.intel.com/tdx-guest:  110
...
Allocated resources:
  (Total limits may be over 100 percent, i.e., overcommitted.)
  Resource                 Requests     Limits
  --------                 --------     ------
  cpu                      1250m (15%)  600m (7%)
  memory                   510Mi (6%)   690Mi (9%)
  ephemeral-storage        0 (0%)       0 (0%)
  hugepages-1Gi            0 (0%)       0 (0%)
  hugepages-2Mi            0 (0%)       0 (0%)
  tdx.intel.com/tdx-guest  0            0
...
```
TDX resource is show in capacity and usage tables


## 4. udev config
setup ccnp user group in host node:
```
groupadd ccnp
```

follow udev rule is provisioned:
```
/etc/udev/rules.d/90-tdx.rules
SUBSYSTEM=="misc",KERNEL=="tdx-guest",MODE="0660",GROUP="ccnp"
```

any user in ccnp group can access /dev/tdx-guest with read nad write permission:
```
ls -l /dev/tdx-guest
crw-rw---- 1 root ccnp 10, 126 Jun  2 02:09 /dev/tdx-guest
```

## 5. deploy CCNP quote service

### get source code and build image
```
git clone https://github.com/intel-sandbox/cse-ccnp.git
docker build -t docker.io/intel/intel-tdx-plugin:v1 -f container/quote-server/Dockerfile .
docker save -o deviceplugin-tdx.tar deviceplugin-tdx:v1
ctr -n k8s.io image import deviceplugin-tdx.tar
```

### deploy quote service daemonset

```
cd cse-ccnp
kubectl service/quote-server/deployment/quote_server-daemonset.yaml
```
Note:
- the daemonset deploys on node with label `intel.feature.node.kubernetes.io/tdx: "true"`
- it requires TDX resource: `tdx.intel.com/tdx-guest: 1`
- it use `/usr/bin/grpc_health_probe` as liveness and readiness probe
- `tolerations` is used for on node cluster, can be removed on nomarl cluster

### check deploy result
```
kubectl get all -n ccnp
NAME                     READY   STATUS    RESTARTS        AGE
pod/quote-server-h57hr   1/1     Running   3 (4m49s ago)   37h

NAME                    TYPE        CLUSTER-IP      EXTERNAL-IP   PORT(S)    AGE
service/qoute-service   ClusterIP   10.110.177.34   <none>        6789/TCP   37h

NAME                          DESIRED   CURRENT   READY   UP-TO-DATE   AVAILABLE   NODE SELECTOR                               AGE
daemonset.apps/quote-server   1         1         1       1            1           intel.feature.node.kubernetes.io/tdx=true   37h
```

### access the quote service:
`grpcurl -d '{"report_data": "MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4"}' -plaintext qoute-service.ccnp:6789  quoteserver.GetQuote.GetQuote`

```{
  "quote": "\"BAACAIEAAAAAAAAAk5pyM/ecTKmUCg2zlX8GB6P8pz1eLkNLuYzlFq7gmQ4AAAAABAAGAAAAAAAAAAAAAAAAAEj6aZSdsIAC7oQlKEf1cpiLHW5WjsE1P2TLbA/ZBTdfaa2VnA6vd0escKOSeJMCoQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEAAAAADnAgYAAAAAAKSgAzRsWhmm/SUEcehyvQcdjJLXQxq9pGNBeAihc4OqDUKYeBS8kvX1nGBEtnf1FAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAOzTrhxKaZL/tz1j5UnYywEl8++4LsAiT4esLkwPtQG0hqwOS8gUO9dZCuRoBPpx7yxsuL/roStq3UMGZb72940BJ8GuZYyl8oJVXbcVxBG2fcOUc6SALFsb/ByxrdBKcAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAADEyMzQ1Njc4MTIzNDU2NzgxMjM0NTY3ODEyMzQ1Njc4MTIzNDU2NzgxMjM0NTY3OAAAAAAAAAAAAAAAAAAAAADMEAAAqBKc+EHhGy+8o9RvUv7CFq8R6VhxJqzqHGB8Sg4jGzjouuLGwm+vWCi8SB3OBhffFkj4X7/XadspSRm/GlmWDCY1oRIxb7dVmrMVrWdmmfldJoB8RvpN7HOs/g788NOgFoemd5F95mGdNVJ3v0ppPvgJQ5ryby6SOYHAsL25dbkGAEYQAAAGBhMVA/8ABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAVAAAAAAAAAOcAAAAAAAAAOWseNYAkJ5SHxHr5xWG93BUlhjmq0t2vdgCQ70Y/C4QAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANyeKnxvlI8XR040p/xD7QMPfBVj8bq932NAyC4OVKjFAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgAEAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAOYzIRlhSwSQeVYIlVCCAgcDb+K5pY0hOxqAEm1vN5p/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABTa87IlHfMMVcMLwVc/wiRRNnC64S5qoxH2VzfZ3Y1wjMZ9A6+fAxBlJO+1Y1dF8L+Csq5Aj8fi8oNriu0ahBCIAAAAQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHwUAXg4AAC0tLS0tQkVHSU4gQ0VSVElGSUNBVEUtLS0tLQpNSUlFOERDQ0JKYWdBd0lCQWdJVWR2clZDNnpJTFRvbGI0ZEt5RGhsbFlaUEc1RXdDZ1lJS29aSXpqMEVBd0l3CmNERWlNQ0FHQTFVRUF3d1pTVzUwWld3Z1UwZFlJRkJEU3lCUWJHRjBabTl5YlNCRFFURWFNQmdHQTFVRUNnd1IKU1c1MFpXd2dRMjl5Y0c5eVlYUnBiMjR4RkRBU0JnTlZCQWNNQzFOaGJuUmhJRU5zWVhKaE1Rc3dDUVlEVlFRSQpEQUpEUVRFTE1Ba0dBMVVFQmhNQ1ZWTXdIaGNOTWpNd05URTJNRGd5TXpJd1doY05NekF3TlRFMk1EZ3lNekl3CldqQndNU0l3SUFZRFZRUUREQmxKYm5SbGJDQlRSMWdnVUVOTElFTmxjblJwWm1sallYUmxNUm93R0FZRFZRUUsKREJGSmJuUmxiQ0JEYjNKd2IzSmhkR2x2YmpFVU1CSUdBMVVFQnd3TFUyRnVkR0VnUTJ4aGNtRXhDekFKQmdOVgpCQWdNQWtOQk1Rc3dDUVlEVlFRR0V3SlZVekJaTUJNR0J5cUdTTTQ5QWdFR0NDcUdTTTQ5QXdFSEEwSUFCUHZQClNtKzJtU1R2TzE0RkhpOXd3K05qYUhzazhyVHFQQ0xEMDZ3MmtJVE9yb0RYSmN5NDBMbHRZemFBZ3JXR2FsWFoKTy9GY3cxc0padDZZdFNRVHlyU2pnZ01NTUlJRENEQWZCZ05WSFNNRUdEQVdnQlNWYjEzTnZSdmg2VUJKeWRUMApNODRCVnd2ZVZEQnJCZ05WSFI4RVpEQmlNR0NnWHFCY2hscG9kSFJ3Y3pvdkwyRndhUzUwY25WemRHVmtjMlZ5CmRtbGpaWE11YVc1MFpXd3VZMjl0TDNObmVDOWpaWEowYVdacFkyRjBhVzl1TDNZMEwzQmphMk55YkQ5allUMXcKYkdGMFptOXliU1psYm1OdlpHbHVaejFrWlhJd0hRWURWUjBPQkJZRUZEUGs4eit4L0JtVkw5UDVJTkRaNlhlUwpTOHR4TUE0R0ExVWREd0VCL3dRRUF3SUd3REFNQmdOVkhSTUJBZjhFQWpBQU1JSUNPUVlKS29aSWh2aE5BUTBCCkJJSUNLakNDQWlZd0hnWUtLb1pJaHZoTkFRMEJBUVFRUGYyUXdCNHRTYzhyRmxvZGJJQzlCVENDQVdNR0NpcUcKU0liNFRRRU5BUUl3Z2dGVE1CQUdDeXFHU0liNFRRRU5BUUlCQWdFRk1CQUdDeXFHU0liNFRRRU5BUUlDQWdFRgpNQkFHQ3lxR1NJYjRUUUVOQVFJREFnRUNNQkFHQ3lxR1NJYjRUUUVOQVFJRUFnRUNNQkFHQ3lxR1NJYjRUUUVOCkFRSUZBZ0VETUJBR0N5cUdTSWI0VFFFTkFRSUdBZ0VCTUJBR0N5cUdTSWI0VFFFTkFRSUhBZ0VBTUJBR0N5cUcKU0liNFRRRU5BUUlJQWdFRE1CQUdDeXFHU0liNFRRRU5BUUlKQWdFQU1CQUdDeXFHU0liNFRRRU5BUUlLQWdFQQpNQkFHQ3lxR1NJYjRUUUVOQVFJTEFnRUFNQkFHQ3lxR1NJYjRUUUVOQVFJTUFnRUFNQkFHQ3lxR1NJYjRUUUVOCkFRSU5BZ0VBTUJBR0N5cUdTSWI0VFFFTkFRSU9BZ0VBTUJBR0N5cUdTSWI0VFFFTkFRSVBBZ0VBTUJBR0N5cUcKU0liNFRRRU5BUUlRQWdFQU1CQUdDeXFHU0liNFRRRU5BUUlSQWdFTE1COEdDeXFHU0liNFRRRU5BUUlTQkJBRgpCUUlDQXdFQUF3QUFBQUFBQUFBQU1CQUdDaXFHU0liNFRRRU5BUU1FQWdBQU1CUUdDaXFHU0liNFRRRU5BUVFFCkJnQ0Fid1VBQURBUEJnb3Foa2lHK0UwQkRRRUZDZ0VCTUI0R0NpcUdTSWI0VFFFTkFRWUVFQUxFbzJLdDd4d3QKNmhQZGdZekRNMFl3UkFZS0tvWklodmhOQVEwQkJ6QTJNQkFHQ3lxR1NJYjRUUUVOQVFjQkFRSC9NQkFHQ3lxRwpTSWI0VFFFTkFRY0NBUUVBTUJBR0N5cUdTSWI0VFFFTkFRY0RBUUgvTUFvR0NDcUdTTTQ5QkFNQ0EwZ0FNRVVDCklBTURNUDNSaUJOYVpuM2NLUjducFVxNDFkTm1HNzIzZlFYcWlJVTU0U09KQWlFQSsrZW9Ta1ZOa2NnbERLZncKaDNDbGx6UzNway9hSGhYNjZDUjc1TllJanpnPQotLS0tLUVORCBDRVJUSUZJQ0FURS0tLS0tCi0tLS0tQkVHSU4gQ0VSVElGSUNBVEUtLS0tLQpNSUlDbGpDQ0FqMmdBd0lCQWdJVkFKVnZYYzI5RytIcFFFbkoxUFF6emdGWEM5NVVNQW9HQ0NxR1NNNDlCQU1DCk1HZ3hHakFZQmdOVkJBTU1FVWx1ZEdWc0lGTkhXQ0JTYjI5MElFTkJNUm93R0FZRFZRUUtEQkZKYm5SbGJDQkQKYjNKd2IzSmhkR2x2YmpFVU1CSUdBMVVFQnd3TFUyRnVkR0VnUTJ4aGNtRXhDekFKQmdOVkJBZ01Ba05CTVFzdwpDUVlEVlFRR0V3SlZVekFlRncweE9EQTFNakV4TURVd01UQmFGdzB6TXpBMU1qRXhNRFV3TVRCYU1IQXhJakFnCkJnTlZCQU1NR1VsdWRHVnNJRk5IV0NCUVEwc2dVR3hoZEdadmNtMGdRMEV4R2pBWUJnTlZCQW9NRVVsdWRHVnMKSUVOdmNuQnZjbUYwYVc5dU1SUXdFZ1lEVlFRSERBdFRZVzUwWVNCRGJHRnlZVEVMTUFrR0ExVUVDQXdDUTBFeApDekFKQmdOVkJBWVRBbFZUTUZrd0V3WUhLb1pJemowQ0FRWUlLb1pJemowREFRY0RRZ0FFTlNCLzd0MjFsWFNPCjJDdXpweHc3NGVKQjcyRXlER2dXNXJYQ3R4MnRWVExxNmhLazZ6K1VpUlpDbnFSN3BzT3ZncUZlU3hsbVRsSmwKZVRtaTJXWXozcU9CdXpDQnVEQWZCZ05WSFNNRUdEQVdnQlFpWlF6V1dwMDBpZk9EdEpWU3YxQWJPU2NHckRCUwpCZ05WSFI4RVN6QkpNRWVnUmFCRGhrRm9kSFJ3Y3pvdkwyTmxjblJwWm1sallYUmxjeTUwY25WemRHVmtjMlZ5CmRtbGpaWE11YVc1MFpXd3VZMjl0TDBsdWRHVnNVMGRZVW05dmRFTkJMbVJsY2pBZEJnTlZIUTRFRmdRVWxXOWQKemIwYjRlbEFTY25VOURQT0FWY0wzbFF3RGdZRFZSMFBBUUgvQkFRREFnRUdNQklHQTFVZEV3RUIvd1FJTUFZQgpBZjhDQVFBd0NnWUlLb1pJemowRUF3SURSd0F3UkFJZ1hzVmtpMHcraTZWWUdXM1VGLzIydWFYZTBZSkRqMVVlCm5BK1RqRDFhaTVjQ0lDWWIxU0FtRDV4a2ZUVnB2bzRVb3lpU1l4ckRXTG1VUjRDSTlOS3lmUE4rCi0tLS0tRU5EIENFUlRJRklDQVRFLS0tLS0KLS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSUNqekNDQWpTZ0F3SUJBZ0lVSW1VTTFscWROSW56ZzdTVlVyOVFHemtuQnF3d0NnWUlLb1pJemowRUF3SXcKYURFYU1CZ0dBMVVFQXd3UlNXNTBaV3dnVTBkWUlGSnZiM1FnUTBFeEdqQVlCZ05WQkFvTUVVbHVkR1ZzSUVOdgpjbkJ2Y21GMGFXOXVNUlF3RWdZRFZRUUhEQXRUWVc1MFlTQkRiR0Z5WVRFTE1Ba0dBMVVFQ0F3Q1EwRXhDekFKCkJnTlZCQVlUQWxWVE1CNFhEVEU0TURVeU1URXdORFV4TUZvWERUUTVNVEl6TVRJek5UazFPVm93YURFYU1CZ0cKQTFVRUF3d1JTVzUwWld3Z1UwZFlJRkp2YjNRZ1EwRXhHakFZQmdOVkJBb01FVWx1ZEdWc0lFTnZjbkJ2Y21GMAphVzl1TVJRd0VnWURWUVFIREF0VFlXNTBZU0JEYkdGeVlURUxNQWtHQTFVRUNBd0NRMEV4Q3pBSkJnTlZCQVlUCkFsVlRNRmt3RXdZSEtvWkl6ajBDQVFZSUtvWkl6ajBEQVFjRFFnQUVDNm5Fd01ESVlaT2ovaVBXc0N6YUVLaTcKMU9pT1NMUkZoV0dqYm5CVkpmVm5rWTR1M0lqa0RZWUwwTXhPNG1xc3lZamxCYWxUVll4RlAyc0pCSzV6bEtPQgp1ekNCdURBZkJnTlZIU01FR0RBV2dCUWlaUXpXV3AwMGlmT0R0SlZTdjFBYk9TY0dyREJTQmdOVkhSOEVTekJKCk1FZWdSYUJEaGtGb2RIUndjem92TDJObGNuUnBabWxqWVhSbGN5NTBjblZ6ZEdWa2MyVnlkbWxqWlhNdWFXNTAKWld3dVkyOXRMMGx1ZEdWc1UwZFlVbTl2ZEVOQkxtUmxjakFkQmdOVkhRNEVGZ1FVSW1VTTFscWROSW56ZzdTVgpVcjlRR3prbkJxd3dEZ1lEVlIwUEFRSC9CQVFEQWdFR01CSUdBMVVkRXdFQi93UUlNQVlCQWY4Q0FRRXdDZ1lJCktvWkl6ajBFQXdJRFNRQXdSZ0loQU9XLzVRa1IrUzlDaVNEY05vb3dMdVBSTHNXR2YvWWk3R1NYOTRCZ3dUd2cKQWlFQTRKMGxySG9NcytYbzVvL3NYNk85UVd4SFJBdlpVR09kUlE3Y3ZxUlhhcUk9Ci0tLS0tRU5EIENFUlRJRklDQVRFLS0tLS0KAA==\""
}
```

