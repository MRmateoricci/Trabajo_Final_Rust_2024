#![cfg_attr(not(feature = "std"), no_std, no_main)]
//en este contrato tuvimos que usar varias implementaciones y structs del contrato anterior para hacer los test del reporte ,
// por lo cual aca quizas no saltan en el coverage pero en el sistema de votacion esta todo testeado. 

#[ink::contract]
mod reporte {
    use sistema::SistemaRef;
    use ink::prelude::vec::Vec;
    use ink::prelude::string::String;
    use ink::prelude::collections::BTreeMap;

    #[ink(storage)]
    pub struct Reporte {
        sistema: SistemaRef,
    }
        
    impl Reporte {

        #[ink(constructor)]
        #[cfg(not(test))]
        pub fn new(s:SistemaRef) -> Reporte{
            Reporte{
                sistema:s
            }
        }

        #[ink(message)]
        #[cfg(not(test))]
        pub fn ver_resultados(&self, id_de_votacion:i32){
            let momento = self.env().block_timestamp();
            self.ver_resultados_impl(id_de_votacion, momento);
        }

        #[cfg(not(test))] 
        pub fn ver_resultados_impl(&self, id_de_votacion:i32, momento:Timestamp){ 
            if let Some(v) = self.sistema.get_votacion(id_de_votacion){ // si existe la votacion
                if v.finalizo(momento){ // si termino 
                    v.get_reporte(self.sistema.get_usuarios()); //llama a reporte y le pasa los usuarios para obtener los nombres y apellidos , porque en las votaciones solo hay account id
                }else{
                    panic!("LA VOTACION: {} NO TERMINO ",id_de_votacion); // si la vot no termino panic
                }  
            }else{
                panic!("NO EXISTE VOTACION DE ID: {}",id_de_votacion); // no existe la votacion
            }
        }

    }
    
    #[cfg(test)]
      mod tests {
        use super::*;
        use ink::env::test;

        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        #[cfg(test)]
        struct Reporte{ // definimos una estructura de reporte para poder utilzarla en los test
            sistema:Mocksistema
        }
        #[cfg(test)]
        impl Reporte{ // implementamos las funciones asociadas que queremos testear
            fn new (sistema:Mocksistema) -> Reporte{
                Reporte{
                    sistema
                }
            }
            fn ver_resultados_impl(&self, id_de_votacion:i32, momento:Timestamp){
                if let Some(v) = self.sistema.get_votacion(id_de_votacion){
                    if v.finalizo(momento){
                        v.get_reporte(self.sistema.get_usuarios());
                    }else{
                        panic!("LA VOTACION: {} NO TERMINO ",id_de_votacion);
                    }  
                }else{
                    panic!("NO EXISTE VOTACION DE ID: {}",id_de_votacion);
                }
            }
        }
        struct Mocksistema{ // simplficamos el struct para testear lo que realmente queremos
            usuarios_reg: Vec<Usuario>, 
            votaciones:Vec<Votacion>, 
            admin:AccountId,
        }

        impl Mocksistema{ // declaramos las funciones que necesitamos para testear el reporte
            fn new(id:AccountId) -> Mocksistema{
               Mocksistema{
                    usuarios_reg:Vec::new(), votaciones:Vec::new(), admin:id
               }     
            }

            fn get_votacion(&self, id_de_votacion: i32)->Option<Votacion>{
                if let Some(vot) = self.votaciones.iter().find(|v|v.id==id_de_votacion){
                    return Some(vot.clone());
                }
                None
            }

            fn get_usuarios(&self) -> Vec<Usuario>{
                self.usuarios_reg.clone()
            }
            fn crear_votacion_impl(&mut self, id:i32, puesto:String,fecha_inicio:Fecha,fecha_fin:Fecha, caller:AccountId){ 
                if !fecha_inicio.es_fecha_valida() | !fecha_fin.es_fecha_valida() | !fecha_fin.es_mayor(&fecha_inicio) {
                    panic!("FECHA INVALIDA");
                }
                if caller == self.admin {  //solo el administrador puede crear votaciones
                    if !self.votaciones.iter().any(|v|v.id==id){  //no se tiene que poder crear dos votaciones con el mismo id
                        let v = Votacion::new(id, puesto, fecha_inicio.to_timestamp(),fecha_fin.to_timestamp());
                        self.votaciones.push(v);       
                        ink::env::debug_println!("fecha inicio: {:?} timestamp: {}",fecha_inicio,fecha_inicio.to_timestamp().wrapping_sub(86_400_000)); //asi comienza ese dia a las 00:00
                        ink::env::debug_println!("fecha fin: {:?} timestamp: {}",fecha_fin,fecha_fin.to_timestamp().wrapping_sub(1));  //asi termina ese dia a las 23:59:59.999
                    }
    
                }
                    
            }
        }
        #[derive(Clone)]
        pub struct Votacion{ // votacion los importamos entero porque necesitamos todos los campos
            id:i32,
            puesto:String,
            candidatos:Vec<AccountId>,
            votantes: Vec<AccountId>,
            votos: BTreeMap<AccountId,u32>,    // hashmap con accountid de candidato
            votaron: Vec<AccountId>,
            fecha_inicio:Timestamp,
            fecha_fin:Timestamp,
        }
        impl Votacion{
            pub fn new(id:i32,puesto:String, fecha_inicio:Timestamp, fecha_fin:Timestamp)-> Votacion{
                Votacion {
                    id, puesto, candidatos:Vec::new(),votantes:Vec::new(), votos:BTreeMap::new(), votaron:Vec::new(),fecha_inicio, fecha_fin
                }
            }
            pub fn finalizo(&self, momento:Timestamp)->bool{ 
                momento >= self.fecha_fin
            }
    
            pub fn sumar_candidato(&mut self,accid:AccountId){
                self.candidatos.push(accid);
                self.votos.insert(accid, 0);
            }
    
            pub fn sumar_votante(&mut self,accid:AccountId){
                self.votantes.push(accid);
            }
    
            pub fn sumar_voto(&mut self,pos:usize){
                self.votos.entry(self.candidatos[pos]).and_modify(|c|* c = c.wrapping_add(1));
            }
            pub fn get_reporte(&self , users:Vec<Usuario>){
            
                let total = self.reporte1();
                if total != 0 && self.candidatos.len() > 0  { 
                    ink::env::debug_println!("EL TOTAL DE USUARIOS REGISTRADOS Y APROBADOS PARA ESTA VOTACION SON: {}",total);
                    let porcentaje = self.reporte2(total);
                    if porcentaje != 0.0{
                        ink::env::debug_println!("EL PORCENTAJE DE USUARIOS QUE VOTARON ES:%{}",porcentaje);
                        let ordenado = self.reporte3();
                        for (i,o) in ordenado.iter().enumerate(){
                            let usuario = users.iter().find(| u | u.acc_id == o.0).unwrap(); // podemos hacer unwrap porque sabemos que el usuario existe 
                            ink::env::debug_println!("{}º - {} {}",i.wrapping_add(1) , usuario.nombre , usuario.apellido );
                        }
                    }else{
                        panic!("NO HUBIERON VOTOS");
                    }
                }else{
                    panic!("NO HUBO USUARIOS/CANDIDATOS ACEPTADOS PARA VOTAR");
                }
    
            }
    
            fn reporte1(&self) ->usize{
                self.votantes.len()
            }
    
            fn reporte2(&self, total:usize) ->f64{
                let mut cantvotos: u32=0;
                for v in &self.votos{
                    cantvotos = cantvotos.wrapping_add(*v.1);
                };
                if cantvotos !=0 {
                    (cantvotos as f64 / total as f64 ) * 100.0
                }
                else{
                     0.0
                }
            }
    
            fn reporte3(&self) -> Vec<(AccountId,u32)>{
                let aux = self.votos.clone();
                let mut vec: Vec<(AccountId, u32)> = aux.into_iter().collect(); // creamos un vector para ordenar de mayor a menor cada candidato 
                vec.sort_by(|a, b| b.1.cmp(&a.1));
                vec
            }
        }
        #[derive(Debug,Clone)]
        
        pub struct Fecha{
            pub dia:u32,
            pub mes:u32,
            pub anio:i32
        }
        
        impl Fecha {

            #[allow(unused)]
            fn new(dia:u32 , mes:u32, anio:i32) -> Fecha{
                Fecha{dia,mes,anio}
            }
    
    
            fn es_fecha_valida(&self)->bool{
                if (self.mes>0)&&(self.mes<=12)&&(self.dia>0)&&(self.dia<=31)&&(self.anio>=1970){
                    match self.mes {
                        2=> {
                            if self.is_leap_year(self.anio){
                                return self.dia<=29
                            }
                            else{
                                return self.dia<=28
                            }
                        }
                        1..=12=> {
                            return self.dia<= self.days_in_month(self.mes)
                        }
                        _=> {}
                    }
                }
                false
            }
    
            pub fn to_timestamp(&self) -> Timestamp {
                let days_since_epoch = self.days_since_epoch() as i64;
                let millis_per_day: i64 = 24 * 60 * 60 * 1000;
                days_since_epoch.wrapping_mul(millis_per_day)  as Timestamp
            }
        
            fn days_since_epoch(&self) -> u32 {
                let mut days: u32 = 0;
        
                // Calcular los días desde el Epoch hasta el año actual
                for year in 1970..self.anio {
                    days = days.wrapping_add(if self.is_leap_year(year) { 366 } else { 365 });
                }
        
                // Sumar los días de los meses previos en el año actual
                for month in 1..self.mes {
                    days = days.wrapping_add(self.days_in_month(month));
                }
        
                // Sumar los días del mes actual
                days = days.wrapping_add(self.dia);
        
                days
            }
        
            fn is_leap_year(&self, year: i32) -> bool {
                (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
            }
        
            fn days_in_month(&self, month: u32) -> u32 {
                match month {
                    1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
    
                    4 | 6 | 9 | 11 => 30,
    
                    2 => {
                        if self.is_leap_year(self.anio) {
                            29
                        } else {
                            28
                        }
                    },
    
                    _ => panic!("MES INVALIDO"),
                }
            }
    
            fn es_mayor(&self, otra_fecha: &Fecha) -> bool {
                if self.anio > otra_fecha.anio {
                    true
                } else if self.anio == otra_fecha.anio {
                    if self.mes > otra_fecha.mes {
                        true
                    } else if self.mes == otra_fecha.mes {
                        if self.dia > otra_fecha.dia {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            
        }
        }
        #[derive(Clone)]
        pub struct Usuario{
            nombre:String,
            apellido:String,
            edad:i32,
            dni:i128,
            verificado:bool,
            rol:Option<Rol>,
            acc_id:AccountId
        }
        impl Usuario{
            pub fn new(nombre:String,apellido:String,dni:i128,edad:i32,verificado:bool,rol:Option<Rol>,acc_id:AccountId)->Self{
                Self{nombre,apellido,dni,edad,verificado,rol,acc_id}
            }
        }
        #[derive(Clone)]
        pub enum Rol{
            Votante,
            Candidato,
        }

        #[ink::test]
        fn test_ver_resultados(){

            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

            let mut sist = Mocksistema::new(accounts.alice);
            sist.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
            
            let user1 = Usuario::new("agus".to_string(), "zap".to_string(), 1111, 20, true, Some(Rol::Votante), accounts.bob);
            let user2 = Usuario::new("dan".to_string(), "colli".to_string(), 2222, 20, true, Some(Rol::Votante), accounts.django);
            let user3 = Usuario::new("valen".to_string(), "simo".to_string(), 333, 20, true, Some(Rol::Candidato), accounts.charlie);
            let user4 = Usuario::new("feli".to_string(), "ino".to_string(), 444, 20, true, Some(Rol::Candidato), accounts.frank);

            sist.usuarios_reg.push(user1.clone());
            sist.usuarios_reg.push(user2.clone());
            sist.usuarios_reg.push(user3.clone());
            sist.usuarios_reg.push(user4.clone());
            sist.votaciones[0].sumar_votante(user1.acc_id);
            sist.votaciones[0].sumar_votante(user2.acc_id);
            sist.votaciones[0].sumar_candidato(user3.acc_id);
            sist.votaciones[0].sumar_candidato(user4.acc_id);
            // Agregar votos
            sist.votaciones[0].sumar_voto(0);
            sist.votaciones[0].sumar_voto(0);
    
            let r: Reporte = Reporte::new(sist);
            let m = Fecha::new(2, 1, 2025);
            r.ver_resultados_impl(1, m.to_timestamp());

        }
        #[ink::test]
        #[should_panic]
        fn test_ver_resultados_no_termino(){

            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

            let mut sist = Mocksistema::new(accounts.alice);
            sist.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
            
            let user1 = Usuario::new("agus".to_string(), "zap".to_string(), 1111, 20, true, Some(Rol::Votante), accounts.bob);
            let user2 = Usuario::new("dan".to_string(), "colli".to_string(), 2222, 20, true, Some(Rol::Votante), accounts.django);
            let user3 = Usuario::new("valen".to_string(), "simo".to_string(), 333, 20, true, Some(Rol::Candidato), accounts.charlie);
            let user4 = Usuario::new("feli".to_string(), "ino".to_string(), 444, 20, true, Some(Rol::Candidato), accounts.frank);

            sist.usuarios_reg.push(user1.clone());
            sist.usuarios_reg.push(user2.clone());
            sist.usuarios_reg.push(user3.clone());
            sist.usuarios_reg.push(user4.clone());
            sist.votaciones[0].sumar_votante(user1.acc_id);
            sist.votaciones[0].sumar_votante(user2.acc_id);
            sist.votaciones[0].sumar_candidato(user3.acc_id);
            sist.votaciones[0].sumar_candidato(user4.acc_id);
            // Agregar votos
            sist.votaciones[0].sumar_voto(0);
            sist.votaciones[0].sumar_voto(0);
    
            let r: Reporte = Reporte::new(sist);
            let m = Fecha::new(31, 12, 2024);
            r.ver_resultados_impl(1, m.to_timestamp());

        }
        #[ink::test]
        #[should_panic]
        fn test_ver_resultados_no_existente(){

            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

            let sist = Mocksistema::new(accounts.alice);

            let r: Reporte = Reporte::new(sist);
            let m = Fecha::new(1, 1, 2025);
            r.ver_resultados_impl(1, m.to_timestamp());

        }

        #[ink::test]
        #[should_panic(expected="NO HUBIERON VOTOS")]
        fn test_ver_resultados_sin_votos(){

            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

            let mut sist = Mocksistema::new(accounts.alice);
            sist.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
            
            let user1 = Usuario::new("agus".to_string(), "zap".to_string(), 1111, 20, true, Some(Rol::Votante), accounts.bob);
            let user2 = Usuario::new("dan".to_string(), "colli".to_string(), 2222, 20, true, Some(Rol::Votante), accounts.django);
            let user3 = Usuario::new("valen".to_string(), "simo".to_string(), 333, 20, true, Some(Rol::Candidato), accounts.charlie);
            let user4 = Usuario::new("feli".to_string(), "ino".to_string(), 444, 20, true, Some(Rol::Candidato), accounts.frank);

            sist.usuarios_reg.push(user1.clone());
            sist.usuarios_reg.push(user2.clone());
            sist.usuarios_reg.push(user3.clone());
            sist.usuarios_reg.push(user4.clone());
            sist.votaciones[0].sumar_votante(user1.acc_id);
            sist.votaciones[0].sumar_votante(user2.acc_id);
            sist.votaciones[0].sumar_candidato(user3.acc_id);
            sist.votaciones[0].sumar_candidato(user4.acc_id);

            let r: Reporte = Reporte::new(sist);
            let m = Fecha::new(1, 1, 2025);
            r.ver_resultados_impl(1, m.to_timestamp());
        }
        #[ink::test]
        #[should_panic(expected="NO HUBO USUARIOS/CANDIDATOS ACEPTADOS PARA VOTAR")]
        fn test_ver_resultados_sin_votantes(){

            let accounts = default_accounts();
            test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);

            let mut sist = Mocksistema::new(accounts.alice);
            sist.crear_votacion_impl(1, "Presidente".to_string(),Fecha::new(5, 7, 2024), Fecha::new(1, 1, 2025),accounts.alice);
            
            let user1 = Usuario::new("agus".to_string(), "zap".to_string(), 1111, 20, true, Some(Rol::Votante), accounts.bob);
            let user2 = Usuario::new("dan".to_string(), "colli".to_string(), 2222, 20, true, Some(Rol::Votante), accounts.django);
            let user3 = Usuario::new("valen".to_string(), "simo".to_string(), 333, 20, true, Some(Rol::Candidato), accounts.charlie);
            let user4 = Usuario::new("feli".to_string(), "ino".to_string(), 444, 20, true, Some(Rol::Candidato), accounts.frank);

            sist.usuarios_reg.push(user1.clone());
            sist.usuarios_reg.push(user2.clone());
            sist.usuarios_reg.push(user3.clone());
            sist.usuarios_reg.push(user4.clone());

            let r: Reporte = Reporte::new(sist);
            let m = Fecha::new(1, 1, 2025);
            r.ver_resultados_impl(1, m.to_timestamp());
        }
    //agregamos test dse fecha para llegar al coverage pedido, porque nos salen las lineas en rojo de las funciones de fecha en este contrato   
    #[ink::test]
    fn test_es_fecha_valida() {
        let fecha_valida = Fecha::new(10, 5, 2024);
        assert!(fecha_valida.es_fecha_valida());

        let fecha_invalida = Fecha::new(30, 2, 2021);
        assert!(!fecha_invalida.es_fecha_valida());
    }

    #[ink::test]
    fn test_es_bisiesto() {
        let fecha_bisiesta = Fecha::new(1, 1, 2020);
        assert!(fecha_bisiesta.is_leap_year(fecha_bisiesta.anio));

        let fecha_no_bisiesta = Fecha::new(1, 1, 2021);
        assert!(!fecha_no_bisiesta.is_leap_year(fecha_no_bisiesta.anio));
    }

    #[ink::test]
    fn test_es_mayor() {
        let fecha1 = Fecha::new(10, 5, 2024);
        let fecha2 = Fecha::new(10, 5, 2023);
        assert!(fecha1.es_mayor(&fecha2));

        let fecha3 = Fecha::new(10, 5, 2024);
        let fecha4 = Fecha::new(10, 4, 2024);
        assert!(fecha3.es_mayor(&fecha4));

        let fecha5 = Fecha::new(10, 5, 2024);
        let fecha6 = Fecha::new(9, 5, 2024);
        assert!(fecha5.es_mayor(&fecha6));
    }
}
}
